use crate::driver::Driver;
use crate::error::Result;

pub trait MtdDriver: Driver {
    fn mtd_read(&self, addr: u32, buffer: &mut [u8]) -> Result<()>;
    fn mtd_write(&self, addr: u32, data: &[u8]) -> Result<()>;
    fn mtd_erase(&self, addr: u32, size: u32) -> Result<()>;
    fn size(&self) -> u32;
    fn erase_size(&self) -> u32;
    fn write_size(&self) -> u32;
}
// pub mod sfud;
