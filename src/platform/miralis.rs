//! Running on top of an host Miralis

use core::fmt;

use log::Level;
use miralis_abi::{failure, miralis_log_fmt, success};

use crate::device::clint::CLINT_SIZE;
use crate::device::tester::TEST_DEVICE_SIZE;
use crate::device::VirtDevice;
use crate::platform::{Plat, VIRT_CLINT, VIRT_TEST_DEVICE};
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

    fn debug_print(level: Level, args: fmt::Arguments) {
        miralis_log_fmt(level, args)
    }

    fn exit_success() -> ! {
        success();
    }

    fn exit_failure() -> ! {
        failure();
    }

    fn get_virtual_devices() -> &'static [VirtDevice] {
        VIRT_DEVICES
    }
}
