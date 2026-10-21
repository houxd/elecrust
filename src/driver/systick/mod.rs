use crate::driver::Driver;

pub trait SysTickDriver: Driver {
    fn get_systick_ms(&self) -> u32;
}
