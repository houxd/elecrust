use crate::driver::systick::SysTickDriver;
use crate::driver::tty::dummy_tty::DummyTty;
use crate::driver::tty::TtyDriver;
use crate::driver::Driver;
use crate::executor::Spawner;
use crate::fs::Fs;
use crate::println;
use crate::sys;
use crate::sys::device::Device;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::any::Any;
use core::future::Future;
use core::pin::Pin;

pub trait Board: Any {
    fn as_any(&self) -> &dyn Any;

    /// 返回设备列表, 板卡必须提供, 用于初始化和反初始化
    fn devices(&self) -> Rc<Vec<Device>>;

    /// 返回文件系统设备, 如果Board没有实现fs, 则返回一个空的Fs
    fn fs(&self) -> Rc<Fs> {
        Rc::new(Fs::new(alloc::vec![]))
    }

    /// 指定系统时钟设备名称, 用于sleep
    fn chosen_systick(&self) -> &'static str {
        "systick0"
    }

    /// 指定系统tty设备名称, 用于println!
    fn chosen_tty(&self) -> &'static str {
        "tty0"
    }

    fn init(&self, spawner: Spawner) {
        // 设置TTY设备为DummyTty, 即使Board没有实现TTY设备, 也要保证println!能正常工作, 后面真正使用的TTY设备会在初始化时被设置
        set_tty_dev(Rc::new(DummyTty));

        // 初始化设备
        for device in self.devices().iter() {
            match device.driver().driver_init(&spawner) {
                Ok(_) => {
                    println!("Initialized device: {}", device.name());
                    match device {
                        Device::SysTick(systick) => {
                            if systick.device_name() == self.chosen_systick() {
                                set_systick_dev(systick.clone());
                                println!("Using {} as system systick device.", systick.device_name());
                            }
                        }
                        Device::Tty(tty) => {
                            if tty.device_name() == self.chosen_tty() {
                                set_tty_dev(tty.clone());
                                println!("Using {} as system tty device.", tty.device_name());
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Failed to initialize device {}: {}", device.name(), e);
                }
            }
        }

        // 挂载文件系统
        let fs = self.fs();
        if !fs.fstab().is_empty() {
            for entry in fs.fstab().iter() {
                println!("Mounting file system: {}", entry.mount_point());
                if let Err(e) = fs.mount(entry.mount_point()) {
                    println!("Failed to mount file system {}: {}", entry.mount_point(), e);
                }
            }
        }
    }

    fn deinit<'a>(&'a self, spawner: Spawner) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(async move {
            // 卸载文件系统
            let fs = self.fs();
            if !fs.fstab().is_empty() {
                for entry in fs.fstab().iter() {
                    println!("Unmounting file system: {}", entry.mount_point());
                    if let Err(e) = entry.fs_driver().unmount() {
                        println!(
                            "Failed to unmount file system {}: {}",
                            entry.mount_point(),
                            e
                        );
                    }
                }
            }

            // 反初始化设备
            for device in self.devices().iter() {
                let driver = device.driver();
                if let Err(e) = driver.driver_deinit(&spawner) {
                    println!("Failed to deinitialize device {}: {}", device.name(), e);
                }

                while !driver.driver_deinit_done(&spawner) {
                    sys::yield_now().await;
                }
            }
        })
    }

    fn get_dev_by_name(&self, dev_name: &str) -> Option<Device> {
        for device in self.devices().iter() {
            if device.driver().device_name() == dev_name {
                return Some(device.clone());
            }
        }
        None
    }

    fn get_dev_by_driver(&self, driver: &dyn Driver) -> Option<Device> {
        for device in self.devices().iter() {
            if core::ptr::addr_eq(
                device.driver() as *const dyn Driver,
                driver as *const dyn Driver,
            ) {
                return Some(device.clone());
            }
        }
        None
    }
}

static mut SYSTICK: Option<Rc<dyn SysTickDriver>> = None;

fn set_systick_dev(systick: Rc<dyn SysTickDriver>) {
    unsafe {
        SYSTICK = Some(systick);
    }
}

pub fn get_systick_dev() -> Rc<dyn SysTickDriver> {
    unsafe {
        if let Some(systick) = core::ptr::addr_of_mut!(SYSTICK)
            .as_mut()
            .unwrap_unchecked()
            .as_ref()
        {
            systick.clone()
        } else {
            panic!("SysTick is not initialized before use!");
        }
    }
}

static mut TTY: Option<Rc<dyn TtyDriver>> = None;

fn set_tty_dev(tty: Rc<dyn TtyDriver>) {
    unsafe {
        TTY = Some(tty);
    }
}

pub fn get_tty_dev() -> Rc<dyn TtyDriver> {
    unsafe {
        if let Some(tty) = core::ptr::addr_of_mut!(TTY)
            .as_mut()
            .unwrap_unchecked()
            .as_ref()
        {
            tty.clone()
        } else {
            // 为了在tty设备初始化之前也能正常调用println!而不panic, 必须返回一个TTY, 即使它什么也不做
            Rc::new(DummyTty)
        }
    }
}
