use alloc::string::String;
use crate::{driver::Driver};

// #[async_trait(?Send)]
pub trait UartDriver: Driver {
    fn uart_write(&self, data: &[u8]);
    fn uart_read(&self, data: &mut [u8]) -> usize;
    fn uart_flush(&self);
    fn uart_rx_clear(&self);
    fn uart_rx_available(&self) -> usize;
    fn uart_rx_idle_ticks(&self) -> u32;

    fn uart_putc(&self, c: u8) {
        self.uart_write(&[c]);
    }
    fn uart_getc(&self) -> Option<u8> {
        let mut buf = [0; 1];
        if self.uart_read(&mut buf) == 1 {
            Some(buf[0])
        } else {
            None
        }
    }
    fn uart_write_str(&self, s: &str) {
        self.uart_write(s.as_bytes());
    }
    fn uart_read_str(&self, s: &mut String, max_len: usize) {
        for _ in 0..max_len {
            if let Some(c) = self.uart_getc() {
                s.push(c as char);
            } else {
                break;
            }
        }
    }
}
