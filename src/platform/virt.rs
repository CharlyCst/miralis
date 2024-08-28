//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, ptr};

use log::Level;
use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::config::{
    PLATFORM_NB_HARTS, TARGET_FIRMWARE_ADDRESS, TARGET_STACK_SIZE, TARGET_START_ADDRESS,
};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::{VirtTestDevice, TEST_DEVICE_SIZE};
use crate::device::{self, VirtDevice};
use crate::driver::ClintDriver;
use crate::{_stack_start, _start_address};
// —————————————————————————— Platform Parameters ——————————————————————————— //

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const TEST_MMIO_ADDRESS: usize = 0x100000;
const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;
const CLINT_BASE: usize = 0x2000000;
const TEST_DEVICE_BASE: usize = 0x3000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);

/// The virtual test device.
static VIRT_TEST_DEVICE: VirtTestDevice = VirtTestDevice::new();

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct VirtPlatform {}

impl Platform for VirtPlatform {
    const NB_HARTS: usize = usize::MAX;

    fn name() -> &'static str {
        "QEMU virt"
    }

    fn init() {
        // Serial
        let mut uart = SERIAL_PORT.lock();
        let mut mmio = unsafe { MmioSerialPort::new(SERIAL_PORT_BASE_ADDRESS) };
        mmio.init();
        *uart = Some(mmio);
    }

    fn debug_print(_level: Level, args: fmt::Arguments) {
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

    fn get_miralis_memory_start_and_size() -> (usize, usize) {
        let size: usize;
        // SAFETY: The unsafe block is required to get the address of the stack and start of
        // Miralis, which are external values defined by the linker.
        // We also ensure that `size` is non-negative and within reasonable bounds
        unsafe {
            size = (_stack_start as usize)
                .checked_sub(_start_address as usize)
                .and_then(|diff| diff.checked_add(TARGET_STACK_SIZE * PLATFORM_NB_HARTS))
                .unwrap();
        }

        (MIRALIS_START_ADDR, size.next_power_of_two())
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn create_virtual_devices() -> [VirtDevice; 2] {
        let virtual_clint: device::VirtDevice = VirtDevice {
            start_addr: CLINT_BASE,
            size: CLINT_SIZE,
            name: "CLINT",
            device_interface: &VIRT_CLINT,
        };

        let virtual_test_device: device::VirtDevice = VirtDevice {
            start_addr: TEST_DEVICE_BASE,
            size: TEST_DEVICE_SIZE,
            name: "TEST",
            device_interface: &VIRT_TEST_DEVICE,
        };

        [virtual_clint, virtual_test_device]
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        &CLINT_MUTEX
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
