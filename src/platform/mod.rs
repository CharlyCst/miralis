pub mod virt;

use core::fmt;

use spin::Mutex;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::driver::ClintDriver;
use crate::{config, device, logger};

/// Export the current platform.
/// For now, only QEMU's Virt board is supported
pub type Plat = virt::VirtPlatform;

pub trait Platform {
    fn init();
    fn debug_print(args: fmt::Arguments);
    fn exit_success() -> !;
    fn exit_failure() -> !;
    fn create_clint_device() -> device::VirtDevice;
    fn get_clint() -> &'static Mutex<ClintDriver>;

    /// Load the firmware (virtual M-mode software) and return its address.
    fn load_firmware() -> usize;

    /// Return the number of PMPs of the platform.
    fn get_nb_pmp() -> usize;

    /// Returns the start and size of Mirage's own memory.
    fn get_mirage_memory_start_and_size() -> (usize, usize);

    /// Return maximum valid address
    fn get_max_valid_address() -> usize;

    const HAS_S_MODE: bool = config::VCPU_S_MODE;
}

pub fn init() {
    Plat::init();
    logger::init();

    // Trap handler
    Arch::init();
}
