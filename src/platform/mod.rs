mod miralis;
mod parameters;
mod utils;
pub mod virt;
pub mod visionfive2;

use core::fmt;

use log::Level;
use spin::Mutex;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::driver::ClintDriver;
use crate::{config, device, logger};

/// Export the current platform.
/// For now, only QEMU's Virt board is supported
#[cfg(not(any(feature = "platform_visionfive2", feature = "platform_miralis")))]
pub type Plat = virt::VirtPlatform;

#[cfg(all(feature = "platform_visionfive2", not(feature = "platform_miralis")))]
pub type Plat = visionfive2::VisionFive2Platform;

#[cfg(feature = "platform_miralis")]
pub type Plat = miralis::MiralisPlatform;

pub trait Platform {
    fn name() -> &'static str;
    fn init();
    fn debug_print(level: Level, args: fmt::Arguments);
    fn exit_success() -> !;
    fn exit_failure() -> !;
    fn create_virtual_devices() -> [device::VirtDevice; 2];
    fn create_passthrough_device() -> device::VirtDevice;
    fn get_clint() -> &'static Mutex<ClintDriver>;

    /// Load the firmware (virtual M-mode software) and return its address.
    fn load_firmware() -> usize;

    /// Returns the start and size of Miralis's own memory.
    fn get_miralis_memory_start_and_size() -> (usize, usize);

    /// Return maximum valid address
    fn get_max_valid_address() -> usize;

    const HAS_S_MODE: bool = config::VCPU_S_MODE;
    const NB_HARTS: usize;
}

pub fn init() {
    Plat::init();
    logger::init();

    // Trap handler
    Arch::init();
}
