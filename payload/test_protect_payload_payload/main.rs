//! Test protect payload policy
//!
//! This payload serve as test payload for the protect payload policy. It must be used with the firmware test_protect_payload_firmware only.
//! These two components together make sure we enforce the protect payload policy correctly.
#![no_std]
#![no_main]
#![feature(start)]

// ———————————————————————————————— Guest OS ———————————————————————————————— //

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Say hello
    log::info!("Hello from test protect payload payload");
    // and exit
    success();
}
