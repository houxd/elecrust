use crate::driver::gpio::{GpioDirection, GpioDriver};
use crate::driver::Driver;
use crate::error::Result;
use crate::executor::Spawner;

pub struct DummyGpio {
    dev_name: &'static str,
}

impl DummyGpio {
    pub fn new(dev_name: &'static str) -> Self {
        Self { dev_name }
    }
}

impl Driver for DummyGpio {
    fn driver_init(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }
    fn driver_deinit(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }
    fn device_name(&self) -> &'static str {
        self.dev_name
    }
}

impl GpioDriver for DummyGpio {
    fn gpio_write(&self, _high: bool) {}
    fn gpio_read(&self) -> bool {
        false
    }
    fn gpio_direction(&self, _direction: GpioDirection) {}
}
