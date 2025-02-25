//! Running on top of an host Miralis

use core::fmt;

use log::Level;
use miralis_abi::{failure, miralis_log_fmt, success};
use spin::Mutex;

use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_START_ADDRESS};
use crate::device::clint::{VirtClint, CLINT_SIZE};
use crate::device::tester::TEST_DEVICE_SIZE;
use crate::device::VirtDevice;
use crate::driver::clint::ClintDriver;
use crate::platform::{Plat, CLINT_MUTEX, VIRT_CLINT, VIRT_TEST_DEVICE};
use crate::Platform;

// ———————————————————————————— Platform Devices ———————————————————————————— //

/// The list of virtual devices exposed on the platform.
static VIRT_DEVICES: &[VirtDevice; 2] = &[
    VirtDevice {
        start_addr: Plat::CLINT_BASE,
        size: CLINT_SIZE,
        name: "CLINT",
        device_interface: &VIRT_CLINT,
    },
    VirtDevice {
        start_addr: Plat::TEST_DEVICE_BASE,
        size: TEST_DEVICE_SIZE,
        name: "TEST",
        device_interface: &VIRT_TEST_DEVICE,
    },
];

// ———————————————————————————————— Platform ———————————————————————————————— //

pub struct MiralisPlatform {}

impl Platform for MiralisPlatform {
    const NB_HARTS: usize = usize::MAX;
    const NB_VIRT_DEVICES: usize = 2;

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
        TARGET_FIRMWARE_ADDRESS
    }

    fn get_miralis_start() -> usize {
        TARGET_START_ADDRESS
    }

    fn get_max_valid_address() -> usize {
        usize::MAX
    }

    fn get_virtual_devices() -> &'static [VirtDevice] {
        VIRT_DEVICES
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        &CLINT_MUTEX
    }

    fn get_vclint() -> &'static VirtClint {
        &VIRT_CLINT
    }
}
