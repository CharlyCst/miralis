//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, ptr};

use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::device::{self, VirtClint};

// —————————————————————————— Platform parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const TEST_MMIO_ADDRESS: usize = 0x100000;
const MIRAGE_START_ADDR: usize = 0x80000000;
const FIRMWARE_START_ADDR: usize = 0x80200000;
const CLINT_BASE: usize = 0x2000000;

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);
static mut VIRT_CLINT: Option<VirtClint> = None;
static mut CLINT_MUTEX: Option<Mutex<VirtClint>> = None;
// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VirtPlatform {}

impl Platform for VirtPlatform {
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
        exit_qemu(true)
    }

    fn exit_failure() -> ! {
        exit_qemu(false)
    }

    fn load_firmware() -> usize {
        // We directly load the firmware from QEMU, nothing to do here.
        FIRMWARE_START_ADDR
    }

    fn get_nb_pmp() -> usize {
        16
    }

    fn get_mirage_memory_start_and_size() -> (usize, usize) {
        let size = FIRMWARE_START_ADDR - MIRAGE_START_ADDR;
        (MIRAGE_START_ADDR, size)
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn create_clint_device() -> device::Device {
        let virt_clint = unsafe {
            if let Some(existing_clint) = &VIRT_CLINT {
                existing_clint
            } else {
                VIRT_CLINT = Some(VirtClint::new());
                VIRT_CLINT.as_ref().unwrap()
            }
        };

        let clint_mutex = unsafe {
            if let Some(existing_mutex) = &CLINT_MUTEX {
                existing_mutex
            } else {
                CLINT_MUTEX = Some(Mutex::new(virt_clint.clone()));
                CLINT_MUTEX.as_ref().unwrap()
            }
        };

        device::Device {
            start_addr: CLINT_BASE,
            size: device::CLINT_SIZE,
            name: "CLINT",
            device_interface: Some(clint_mutex),
        }
    }

    fn get_clint_base() -> usize {
        CLINT_BASE
    }
}

fn exit_qemu(success: bool) -> ! {
    let code = if success { 0x5555 } else { (1 << 16) | 0x3333 };

    unsafe {
        let mmio_addr = TEST_MMIO_ADDRESS as *mut i32;
        ptr::write_volatile(mmio_addr, code);
    }

    // Loop forever if shutdown failed
    loop {
        core::hint::spin_loop();
    }
}
