use crate::driver::Driver;

pub enum GpioDirection {
    Input,
    Output(bool),
}

pub trait GpioDriver: Driver {
    fn gpio_direction(&self, direction: GpioDirection);
    fn gpio_write(&self, value: bool);
    fn gpio_read(&self) -> bool;

    fn gpio_toggle(&self) {
        let current = self.gpio_read();
        self.gpio_write(!current);
    }
    fn gpio_set_high(&self) {
        self.gpio_write(true);
    }
    fn gpio_set_low(&self) {
        self.gpio_write(false);
    }
}

pub mod dummy_gpio;
