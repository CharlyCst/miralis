//! Running on top of an host Miralis

use core::fmt;

use log::Level;
use miralis_abi::{failure, miralis_log_fmt, success};
use spin::Mutex;

use crate::config::{TARGET_FIRMWARE_ADDRESS, TARGET_PAYLOAD_ADDRESS};
use crate::device::VirtDevice;
use crate::driver::ClintDriver;
use crate::platform::utils::{
    compute_miralis_memory_start_and_size, create_passthrough_device_default,
    create_virtual_devices_default, get_clint_default, get_max_valid_address_default,
};
use crate::Platform;

// —————————————————————————— Platform Parameters ——————————————————————————— //

const MIRALIS_START_ADDR_MIRALIS: usize = TARGET_FIRMWARE_ADDRESS;
const FIRMWARE_START_ADDR_MIRALIS: usize = TARGET_PAYLOAD_ADDRESS;

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
        FIRMWARE_START_ADDR_MIRALIS
    }

    fn get_miralis_memory_start_and_size() -> (usize, usize) {
        compute_miralis_memory_start_and_size(MIRALIS_START_ADDR_MIRALIS)
    }

    fn get_max_valid_address() -> usize {
        get_max_valid_address_default()
    }

    fn create_virtual_devices() -> [VirtDevice; 2] {
        create_virtual_devices_default()
    }

    fn get_clint() -> &'static Mutex<ClintDriver> {
        get_clint_default()
    }

    fn create_passthrough_device() -> VirtDevice {
        create_passthrough_device_default()
    }
}
