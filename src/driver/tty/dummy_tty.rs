use crate::{
    driver::{tty::TtyDriver, Driver},
    error::Result,
    executor::Spawner,
};

pub struct DummyTty;

impl Driver for DummyTty {
    fn driver_init(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }

    fn driver_deinit(&self, _spawner: &Spawner) -> Result<()> {
        Ok(())
    }

    fn device_name(&self) -> &'static str {
        "dummy_tty"
    }
}

impl TtyDriver for DummyTty {
    fn tty_getc(&self) -> Option<u8> {
        None
    }
    fn tty_putc(&self, _byte: u8) {}
    fn tty_flush(&self) {}
    fn tty_get_break(&self) -> bool {
        false
    }
}
