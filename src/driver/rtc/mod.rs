use crate::driver::Driver;
use crate::error::Result;
use time::UtcDateTime;

pub trait RtcDriver: Driver {
    fn rtc_read(&self) -> Result<UtcDateTime>;
    fn rtc_write(&self, dt: &UtcDateTime) -> Result<()>;
}

pub mod dummy_rtc;
