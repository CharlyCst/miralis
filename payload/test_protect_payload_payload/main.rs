//! Test protect payload policy
//!
//! This payload serve as test payload for the protect payload policy. It must be used with the firmware test_protect_payload_firmware only.
//! These two components together make sure we enforce the protect payload policy correctly.
#![no_std]
#![no_main]
#![feature(start)]

// ———————————————————————————————— Guest OS ———————————————————————————————— //

use core::arch::asm;

use miralis_abi::{lock_payload, log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Say hello
    log::info!("Hello from test protect payload payload");

    // Lock payload to firmware
    lock_payload();

    // Make sure the firmware can't overwrite the set of registers
    assert!(
        test_same_registers_after_trap(),
        "Test same register after trap failed"
    );

    // Make sure the firmware can't overwrite a memory region
    assert!(
        test_same_region_after_trap(),
        "Test same region after trap failed"
    );

    // and exit
    success();
}

fn test_same_registers_after_trap() -> bool {
    let x3: usize;
    unsafe {
        asm!("li t6, 60", "csrw mscratch, zero");
        asm!("", out("t6") x3);
    }

    x3 == 60
}

fn test_same_region_after_trap() -> bool {
    let address: *const usize = 0x80400000 as *const usize;
    let value: usize;

    unsafe {
        value = *address;
    }

    value != 60
}
