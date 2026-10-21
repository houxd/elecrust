use crate::sys::get_systick_dev;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct SleepMsFuture {
    escape: u32,
}

impl Future for SleepMsFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_tick = get_systick_dev().get_systick_ms();
        if current_tick >= self.escape {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[allow(unused)]
pub fn sleep_ms(ms: u32) -> SleepMsFuture {
    unsafe {
        SleepMsFuture {
            escape: ms + get_systick_dev().get_systick_ms(),
        }
    }
}

#[allow(unused)]
#[inline]
pub fn delay_ms(ms: u32) {
    let start = get_systick_dev().get_systick_ms();
    while get_systick_dev().get_systick_ms().wrapping_sub(start) < ms {}
}

pub fn get_systick_ms() -> u32 {
    get_systick_dev().get_systick_ms()
}
