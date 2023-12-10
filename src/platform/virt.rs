//! QEMU Virt board

use core::fmt;
use core::fmt::Write;

use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;

// —————————————————————————— Platform parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VirtPlatform {}

impl Platform for VirtPlatform {
    fn init() {
        let mut uart = SERIAL_PORT.lock();
        let mut mmio = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
        mmio.init();
        *uart = Some(mmio);
    }

    fn debug_print(args: fmt::Arguments) {
        let mut serial_port = SERIAL_PORT.lock();
        if let Some(ref mut serial_port) = serial_port.as_mut() {
            serial_port
                .write_fmt(args)
                .expect("Printing to serial failed")
        };
    }
}
