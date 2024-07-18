//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, hint};

use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::arch::{Arch, Architecture};
use crate::device::{self, VirtClint};
use crate::driver::ClintDriver;

// —————————————————————————— Platform Parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const MIRALIS_START_ADDR: usize = 0x80000000;
const FIRMWARE_START_ADDR: usize = 0x80200000;
const CLINT_BASE: usize = 0x2000000;
const PMP_NUMBER: usize = 8;

// ———————————————————————————— Platform Devices ———————————————————————————— //

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VisionFive2Platform {}

impl Platform for VisionFive2Platform {
    const NB_HARTS: usize = 5;

    fn name() -> &'static str {
        "VisionFive 2 board"
    }

    fn init() {
        // Serial
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

    fn get_nb_pmp() -> usize {
        PMP_NUMBER
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
