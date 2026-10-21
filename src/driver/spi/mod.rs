use crate::driver::{
    // gpio::GpioDriver, 
    Driver};
use crate::error::Result;
use alloc::vec;

pub trait SpiDriver: Driver {
    fn spi_write_read(&self, data: &[u8], buffer: &mut [u8]) -> Result<()>;
    fn spi_cs_activate(&self);
    fn spi_cs_deactivate(&self);

    // fn spi_cs_pin(&self) -> &mut dyn GpioDriver;

    // fn spi_cs_activate(&self) {
    //     self.spi_cs_pin().gpio_set_low();
    // }
    // fn spi_cs_deactivate(&self) {
    //     self.spi_cs_pin().gpio_set_high();
    // }
    fn spi_write(&self, data: &[u8]) -> Result<()> {
        self.spi_write_read(data, &mut [])
    }
    fn spi_read(&self, buffer: &mut [u8]) -> Result<()> {
        self.spi_write_read(&vec![0; buffer.len()], buffer)
    }
}
