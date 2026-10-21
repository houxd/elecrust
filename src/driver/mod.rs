use crate::{error::Result, executor::Spawner};

pub trait Driver {
    fn driver_init(&self, _spawner: &Spawner) -> Result<()>;
    fn driver_deinit(&self, _spawner: &Spawner) -> Result<()>;
    /// 对于一些需要等待的设备, 如I2C, SPI, UART等, 需要等待设备初始化完成后再返回
    /// 如果设备初始化完成, 则返回true, 否则返回false
    /// 如果设备初始化完成, 则返回true, 否则返回false
    fn driver_deinit_done(&self, _spawner: &Spawner) -> bool {
        true
    }
    fn driver_name(&self) -> &'static str {
        core::any::type_name::<Self>().split("::").last().unwrap()
    }
    fn device_name(&self) -> &'static str;
}

pub mod disk;
pub mod gpio;
pub mod i2c;
pub mod mtd;
pub mod rtc;
pub mod spi;
pub mod systick;
pub mod tty;
pub mod uart;
