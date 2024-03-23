pub mod virt;

use core::fmt;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::logger;

/// Export the current platform.
/// For now, only QEMU's Virt board is supported
pub type Plat = virt::VirtPlatform;

pub trait Platform {
    fn init();
    fn debug_print(args: fmt::Arguments);
    fn exit_success() -> !;
    fn exit_failure() -> !;

    /// Load the payload (virtual M-mode software) and return its address.
    fn load_payload() -> usize;

    /// Return the number of PMPs of the platform.
    fn get_nb_pmp() -> usize;

    /// Return maximum valid address
    fn get_max_valid_address() -> usize;
}

pub fn init() {
    Plat::init();
    logger::init();

    // Trap handler
    Arch::init();
}
