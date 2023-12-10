pub mod virt;

use core::fmt;

// Re-export virt platform by default for now
use crate::logger;
use virt::VirtPlatform as CurrentPlatform;

pub trait Platform {
    fn init();
    fn debug_print(args: fmt::Arguments);
    fn exit_success() -> !;
    fn exit_failure() -> !;
}

pub fn init() {
    CurrentPlatform::init();
    logger::init(log::LevelFilter::Info);
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
