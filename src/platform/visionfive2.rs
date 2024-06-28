//! QEMU Virt board

use core::arch::asm;
use core::fmt::Write;
use core::{fmt, hint, ptr};

use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::arch::{Arch, Architecture};
use crate::device::{self, VirtClint};
use crate::driver::ClintDriver;

// —————————————————————————— Platform Parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const MIRALIS_START_ADDR: usize = 0x40000000;
const FIRMWARE_START_ADDR: usize = 0x40200000;
const CLINT_BASE: usize = 0x2000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);
pub static WRITER: Mutex<Writer> = Mutex::new(Writer::new(SERIAL_PORT_BASE_ADDRESS));

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VisionFive2Platform {}

impl Platform for VisionFive2Platform {
    const NB_HARTS: usize = 5;

    fn name() -> &'static str {
        "VisionFive 2 board"
    }

    fn init() {
        let mut writer = WRITER.lock();
        writer.write_char('\n');
        drop(writer);
    }

    fn debug_print(args: fmt::Arguments) {
        let mut writer = WRITER.lock();
        writer.write_fmt(args).unwrap();
        writer.write_str("\r\n").unwrap();
        drop(writer);
    }

    fn exit_success() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    fn exit_failure() -> ! {
        loop {
            Arch::wfi();
            hint::spin_loop();
        }
    }

    fn load_firmware() -> usize {
        FIRMWARE_START_ADDR
    }

    fn get_miralis_memory_start_and_size() -> (usize, usize) {
        let size = FIRMWARE_START_ADDR - MIRALIS_START_ADDR;
        (MIRALIS_START_ADDR, size)
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn create_clint_device() -> device::VirtDevice {
        device::VirtDevice {
            start_addr: CLINT_BASE,
            size: device::CLINT_SIZE,
            name: "CLINT",
            device_interface: &VIRT_CLINT,
        }
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        &CLINT_MUTEX
    }
}

pub struct Writer {
    serial_port_base_addr: usize,
}

impl Writer {
    pub const fn new(serial_port_base_addr: usize) -> Self {
        Writer {
            serial_port_base_addr,
        }
    }

    fn write_char(&mut self, c: char) {
        unsafe {
            ptr::write_volatile(self.serial_port_base_addr as *mut char, c);
            for _n in 1..10001 {
                asm!("nop");
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
