pub mod virt;

use core::fmt;

// Re-export virt platform by default for now
use virt::VirtPlatform as CurrentPlatform;

pub trait Platform {
    fn init();
    fn debug_print(args: fmt::Arguments);
}

pub fn init() {
    CurrentPlatform::init();
}

pub fn debug_print(args: fmt::Arguments) {
    CurrentPlatform::debug_print(args);
}
