use spin::Mutex;
use uart_16550::MmioSerialPort;

use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::VirtClint;
use crate::device::passthrough::PassThroughModule;
use crate::device::tester::VirtTestDevice;
use crate::driver::ClintDriver;

// —————————————————————————— Platform Parameters ——————————————————————————— //

pub const MIRALIS_START_ADDR: usize = TARGET_START_ADDRESS;
pub const FIRMWARE_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;
pub const SERIAL_PORT_BASE_ADDRESS: usize = 0x10000000;
pub const TEST_MMIO_ADDRESS: usize = 0x100000;

pub const PASSTHROUGH_BASE: usize = 0x2000000;
pub const PASSTHROUGH_SIZE: usize = 0x2000000;
pub const CLINT_BASE: usize = 0x2000000;
pub const TEST_DEVICE_BASE: usize = 0x3000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

pub static SERIAL_PORT: Mutex<Option<MmioSerialPort>> = Mutex::new(None);

/// The physical CLINT driver.
///
/// SAFETY: this is the only CLINT device driver that we create, and the platform code does not
/// otherwise access the CLINT.
pub static CLINT_MUTEX: Mutex<ClintDriver> = unsafe { Mutex::new(ClintDriver::new(CLINT_BASE)) };

/// The virtual CLINT device.
pub static VIRT_CLINT: VirtClint = VirtClint::new(&CLINT_MUTEX);

/// The virtual test device.
pub static VIRT_TEST_DEVICE: VirtTestDevice = VirtTestDevice::new();

/// Passthrough module
pub static mut PASS_THROUGH_MODULE: PassThroughModule = PassThroughModule::new(PASSTHROUGH_BASE);
