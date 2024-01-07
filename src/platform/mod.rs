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

    /// Return the initial payload stack address.
    fn stack_address() -> usize;
}

pub fn init() {
    Plat::init();
    logger::init(log::LevelFilter::Info);

    // Trap handler
    Arch::init();
}
