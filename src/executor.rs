use crate::error::Result;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub type TaskId = u16;

pub struct Task {
    id: TaskId,
    name: String,
    future: Pin<Box<dyn Future<Output = Result<()>>>>,
}

impl Task {
    pub fn new(
        id: TaskId,
        name: String,
        future: Pin<Box<dyn Future<Output = Result<()>>>>,
    ) -> Self {
        Self { id, name, future }
    }
}

// 共享的内部状态
struct SharedState {
    tasks: VecDeque<Task>,
    // 存储已经结束的任务的执行结果，供 wait 获取
    exited_tasks: Vec<(TaskId, Result<()>)>,
    next_id_hint: TaskId,
    // 当前正在被 poll 的任务 id。单线程协作式调度下同一时刻只会有一个任务在运行，
    // 因此任务内部可以据此获取"自己的 id"，类似进程获取自身 pid。
    current_task_id: Option<TaskId>,
}

impl SharedState {
    fn next_id(&mut self) -> TaskId {
        let mut candidate = self.next_id_hint;
        let start = candidate;

        loop {
            // 检查当前候选ID是否可用 (不在运行队列，也不在已结束未被收割的队列)
            let is_used = self.tasks.iter().any(|task| task.id == candidate)
                || self.exited_tasks.iter().any(|&(id, _)| id == candidate);

            if !is_used {
                self.next_id_hint = candidate.wrapping_add(1);
                if self.next_id_hint == 0 {
                    self.next_id_hint = 1;
                }
                return candidate;
            }

            candidate = candidate.wrapping_add(1);
            if candidate == 0 {
                candidate = 1;
            }

            if candidate == start {
                panic!("No more available task IDs");
            }
        }
    }
}

// 生产者：负责往队列里派生新任务，以及管理任务（kill, wait）
#[derive(Clone)]
pub struct Spawner {
    state: Rc<RefCell<SharedState>>,
}

impl Spawner {
    /// 派生新任务，类似 Linux 的 fork/exec，返回进程 ID
    pub fn spawn(
        &self,
        name: impl Into<String>,
        future: Pin<Box<dyn Future<Output = Result<()>>>>,
    ) -> TaskId {
        let mut state = self.state.borrow_mut();
        let id = state.next_id();
        state.tasks.push_back(Task::new(id, name.into(), future));
        id
    }

    /// 杀掉一个任务，类似 Linux 的 kill
    /// 在 Rust 中，把 Future 从队列中移除会导致它立刻被 Drop。
    /// 这意味着它占用的所有局部变量和借用都会被安全且立刻释放！
    pub fn kill(&self, id: TaskId) -> bool {
        let mut state = self.state.borrow_mut();
        if let Some(pos) = state.tasks.iter().position(|t| t.id == id) {
            state.tasks.remove(pos); // 核心：从队列移除，自动 Drop 销毁
            true
        } else {
            false
        }
    }

    /// 异步等待一个任务执行完毕，类似 Linux 的 waitpid
    pub fn wait(&self, id: TaskId) -> WaitTaskFuture {
        WaitTaskFuture {
            state: self.state.clone(),
            task_id: id,
        }
    }

    #[allow(unused)]
    pub fn has_tasks(&self) -> bool {
        !self.state.borrow().tasks.is_empty()
    }

    #[allow(unused)]
    pub fn task_count(&self) -> usize {
        self.state.borrow().tasks.len()
    }

    pub fn task_list(&self) -> Vec<(TaskId, String)> {
        self.state
            .borrow()
            .tasks
            .iter()
            .map(|task| (task.id, task.name.clone()))
            .collect()
    }

    pub fn is_running(&self, id: TaskId) -> bool {
        self.state.borrow().tasks.iter().any(|t| t.id == id)
    }

    /// 获取当前正在运行的任务的 id，类似进程获取自身 pid。
    /// 在任务的异步执行流程中调用会返回该任务的 id；
    /// 在任何任务之外（如执行器启动前）调用则返回 `None`。
    pub fn get_task_id(&self) -> Option<TaskId> {
        self.state.borrow().current_task_id
    }
}

/// 自定义的 Future，用于等待目标任务完成
pub struct WaitTaskFuture {
    state: Rc<RefCell<SharedState>>,
    task_id: TaskId,
}

impl Future for WaitTaskFuture {
    type Output = Option<Result<()>>; // None 表示该任务不存在

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();

        // 1. 检查任务是否已经执行完毕
        if let Some(pos) = state
            .exited_tasks
            .iter()
            .position(|&(id, _)| id == self.task_id)
        {
            // 拿走结果，相当于清理了僵尸进程
            let (_, result) = state.exited_tasks.remove(pos);
            return Poll::Ready(Some(result));
        }

        // 2. 检查任务是否还在运行
        if state.tasks.iter().any(|t| t.id == self.task_id) {
            // 还在运行，告诉执行器下个循环再来唤醒我
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            // 不在运行队列，也不在已结束队列，说明是个无效的或者已经被 wait 过的 ID
            Poll::Ready(None)
        }
    }
}

// 消费者：负责从队列中取出任务并执行
pub struct Executor {
    state: Rc<RefCell<SharedState>>,
}

impl Executor {
    pub fn new() -> (Self, Spawner) {
        let state = Rc::new(RefCell::new(SharedState {
            tasks: VecDeque::new(),
            exited_tasks: Vec::new(),
            next_id_hint: 1,
            current_task_id: None,
        }));

        (
            Self {
                state: state.clone(),
            },
            Spawner { state },
        )
    }

    pub fn run(&self) -> ! {
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut context = Context::from_waker(&waker);

        loop {
            let task_opt = self.state.borrow_mut().tasks.pop_front();

            if let Some(mut task) = task_opt {
                // 记录当前任务 id，供任务内部通过 spawner.get_task_id() 获取自身 id。
                // 注意：此处已经把 task 从队列 pop 出来，state 借用已释放，
                // poll 期间任务内部可以安全地再次借用 state。
                self.state.borrow_mut().current_task_id = Some(task.id);
                let poll_result = task.future.as_mut().poll(&mut context);
                self.state.borrow_mut().current_task_id = None;

                match poll_result {
                    Poll::Ready(result) => {
                        // 任务执行完成，将结果存入 exited_tasks 供 wait 函数“收尸”
                        // 如果没人 wait 它，也可以定期清理，或者让它随 Executor 销毁
                        self.state.borrow_mut().exited_tasks.push((task.id, result));
                    }
                    Poll::Pending => {
                        self.state.borrow_mut().tasks.push_back(task);
                    }
                }
            }
        }
    }
}

// 简单的 dummy waker
fn dummy_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    RawWaker::new(core::ptr::null::<()>(), &VTABLE)
}
