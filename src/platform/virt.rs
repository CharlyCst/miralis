//! QEMU Virt board

use core::fmt::Write;
use core::{fmt, ptr};

use log::Level;
use spin::Mutex;
use uart_16550::MmioSerialPort;

use super::Platform;
use crate::config::{PLATFORM_NAME, TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::{VirtTestDevice, TEST_DEVICE_SIZE};
use crate::device::{self, VirtDevice};
use crate::driver::ClintDriver;

const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
const TEST_MMIO_ADDRESS: usize = 0x100000;
const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;
const CLINT_BASE: usize = 0x2000000;
const TEST_DEVICE_BASE: usize = 0x3000000;

// —————————————————————————— Spike Parameters ——————————————————————————— //

/// Symbol used by the Spike simulator.
#[no_mangle]
#[used]
static mut tohost: u64 = 0;

/// Symbol used by the Spike simulator.
#[no_mangle]
#[used]
static mut fromhost: u64 = 0;

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
        match PLATFORM_NAME {
            "spike" => "Spike",
            _ => "QEMU virt",
        }
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
        match PLATFORM_NAME {
            "spike" => exit_spike(true),
            _ => exit_qemu(true),
        }
    }

    fn exit_failure() -> ! {
        match PLATFORM_NAME {
            "spike" => exit_spike(false),
            _ => exit_qemu(false),
        }
    }

    fn load_firmware() -> usize {
        // We directly load the firmware from QEMU, nothing to do here.
        FIRMWARE_START_ADDR
    }

    fn get_miralis_start() -> usize {
        MIRALIS_START_ADDR
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

    fn get_vclint() -> &'static VirtClint {
        &VIRT_CLINT
    }
}

/// Exit the QEMU emulator.
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

/// Exit the spike emulator
fn exit_spike(success: bool) -> ! {
    let code: i32 = if success { 0x1 } else { 0x3 };

    // Requests spike exit by writing exit code to .tohost
    // The write must be volatile to ensure it is not optimized away.
    unsafe {
        ptr::write_volatile(&raw mut tohost, code as u64);
    }

    // Wait until spike shuts down
    loop {
        core::hint::spin_loop();
    }
}
