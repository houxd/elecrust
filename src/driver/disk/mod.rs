use crate::driver::Driver;
use crate::error::Result;

pub trait DiskDriver: Driver {
    fn disk_read(&self, sector: u32, count: u32, buffer: &mut [u8]) -> Result<()>;
    fn disk_write(&self, sector: u32, count: u32, data: &[u8]) -> Result<()>;
}
