use crate::driver::Driver;
use crate::error::Result;

pub trait I2cDriver: Driver {
    fn i2c_write(&self, addr: u16, data: &[u8]) -> Result<()>;
    fn i2c_read(&self, addr: u16, buffer: &mut [u8]) -> Result<()>;
}
