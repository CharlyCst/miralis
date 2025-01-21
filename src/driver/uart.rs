//! # UART Driver
//!
//! This module implements a driver for the UART driver. This driver allows Miralis to communicate with the UART.

use core::fmt::Write;
use core::{fmt, ptr};

pub struct UartDriver {
    serial_port_base_addr: usize,
    size_per_register: usize,
}

impl UartDriver {
    pub const fn new(serial_port_base_addr: usize, size_per_register: usize) -> Self {
        UartDriver {
            serial_port_base_addr,
            size_per_register,
        }
    }

    pub const fn get_register(&mut self, offset: usize) -> usize {
        // Registers are 32 bits wide on the board
        self.serial_port_base_addr + offset * self.size_per_register
    }

    pub(crate) fn write_char(&mut self, c: char) {
        unsafe {
            while self.is_line_busy() {}

            ptr::write_volatile(self.serial_port_base_addr as *mut char, c);
        }
    }

    fn is_line_busy(&mut self) -> bool {
        // Line Status Register
        const LSR_OFFSET: usize = 0x05;
        // Transmit Holding Register Empty
        const LSR_THRE: u8 = 0x20;

        unsafe { ptr::read_volatile(self.get_register(LSR_OFFSET) as *const u8) & LSR_THRE == 0 }
    }
}
impl Write for UartDriver {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
