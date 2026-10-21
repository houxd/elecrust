use crate::driver::gpio::GpioDriver;
use crate::driver::mtd::MtdDriver;
use crate::driver::rtc::RtcDriver;
use crate::driver::spi::SpiDriver;
use crate::driver::systick::SysTickDriver;
use crate::driver::tty::TtyDriver;
use crate::driver::uart::UartDriver;
use crate::driver::Driver;
use crate::fs::FsDriver;
use alloc::rc::Rc;

#[derive(Clone)]
pub enum Device {
    Generic(Rc<dyn Driver>),
    Tty(Rc<dyn TtyDriver>),
    SysTick(Rc<dyn SysTickDriver>),
    Gpio(Rc<dyn GpioDriver>),
    Uart(Rc<dyn UartDriver>),
    Spi(Rc<dyn SpiDriver>),
    Mtd(Rc<dyn MtdDriver>),
    FsDriver(Rc<dyn FsDriver>),
    Rtc(Rc<dyn RtcDriver>),
}

impl Device {
    #[inline]
    pub fn driver(&self) -> &(dyn Driver + 'static) {
        match self {
            Self::Generic(driver) => driver.as_ref(),
            Self::Tty(driver) => driver.as_ref(),
            Self::SysTick(driver) => driver.as_ref(),
            Self::Gpio(driver) => driver.as_ref(),
            Self::Uart(driver) => driver.as_ref(),
            Self::Spi(driver) => driver.as_ref(),
            Self::Mtd(driver) => driver.as_ref(),
            Self::FsDriver(driver) => driver.as_ref(),
            Self::Rtc(driver) => driver.as_ref(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.driver().device_name()
    }
}

impl core::ops::Deref for Device {
    type Target = dyn Driver;

    fn deref(&self) -> &Self::Target {
        self.driver()
    }
}
