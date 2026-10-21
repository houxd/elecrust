use crate::err;
use time::UtcDateTime;
use crate::{
    driver::{rtc::RtcDriver, Driver},
    error::Result,
    executor::Spawner,
    sys,
};

pub struct DummyRtc;

impl Driver for DummyRtc {
    fn driver_init(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }

    fn device_name(&self) -> &'static str {
        "dummy_rtc"
    }
}

impl RtcDriver for DummyRtc {
    fn rtc_read(&self) -> Result<UtcDateTime> {
        let systick = sys::get_systick_ms();
        let utc = UtcDateTime::from_unix_timestamp(systick as i64)
            .map_err(|e| err!("failed to read datetime: {}", e))?;
        Ok(utc)
    }

    fn rtc_write(&self, _utc: &UtcDateTime) -> Result<()> {
        Ok(())
    }
}
