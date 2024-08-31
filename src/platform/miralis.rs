//! Running on top of an host Miralis

use core::fmt;

use log::Level;
use miralis_abi::{failure, miralis_log_fmt, success};
use spin::Mutex;

use crate::config::{
    PLATFORM_NB_HARTS, TARGET_FIRMWARE_ADDRESS, TARGET_PAYLOAD_ADDRESS, TARGET_STACK_SIZE,
};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::{VirtTestDevice, TEST_DEVICE_SIZE};
use crate::device::{self, VirtDevice};
use crate::driver::ClintDriver;
use crate::{Platform, _stack_start, _start_address};
// —————————————————————————— Platform Parameters ——————————————————————————— //

const MIRALIS_START_ADDR: usize = TARGET_FIRMWARE_ADDRESS;
const FIRMWARE_START_ADDR: usize = TARGET_PAYLOAD_ADDRESS;
const CLINT_BASE: usize = 0x2000000;
const TEST_DEVICE_BASE: usize = 0x3000000;

// ———————————————————————————— Platform Devices ———————————————————————————— //

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

pub struct MiralisPlatform {}

impl Platform for MiralisPlatform {
    const NB_HARTS: usize = usize::MAX;

    fn name() -> &'static str {
        "Miralis"
    }

    fn init() {}

    fn debug_print(level: Level, args: fmt::Arguments) {
        miralis_log_fmt(level, args)
    }

    fn exit_success() -> ! {
        success();
    }

    fn exit_failure() -> ! {
        failure();
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
