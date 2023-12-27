pub mod virt;

use core::fmt;

// Re-export virt platform by default for now
use crate::arch::{Arch, Architecture};
use crate::logger;
use virt::VirtPlatform as CurrentPlatform;

pub trait Platform {
    fn init();
    fn debug_print(args: fmt::Arguments);
    fn exit_success() -> !;
    fn exit_failure() -> !;

    /// Load the payload (virtual M-mode software) and return its address.
    fn load_payload() -> usize;
}

pub fn init() {
    CurrentPlatform::init();
    logger::init(log::LevelFilter::Info);

    // Trap handler
    Arch::init();
}

/// Load the payload (virtual M-mode software) and return its address.
pub fn load_payload() -> usize {
    CurrentPlatform::load_payload()
}

pub fn debug_print(args: fmt::Arguments) {
    CurrentPlatform::debug_print(args);
}

pub fn exit_success() -> ! {
    CurrentPlatform::exit_success();
}

pub fn exit_failure() -> ! {
    CurrentPlatform::exit_failure();
}
