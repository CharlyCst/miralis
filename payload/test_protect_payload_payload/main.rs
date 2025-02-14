//! Test protect payload policy
//!
//! This payload serve as test payload for the protect payload policy. It must be used with the firmware test_protect_payload_firmware only.
//! These two components together make sure we enforce the protect payload policy correctly.
#![no_std]
#![no_main]
#![feature(start)]

// ———————————————————————————————— Guest OS ———————————————————————————————— //

use core::arch::asm;

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Say hello
    log::info!("Hello from test protect payload payload");

    // Make sure the ecall parameters goes through
    assert!(test_ecall_rule(), "Ecall test failed");

    // and exit
    success();
}

fn test_ecall_rule() -> bool {
    let ret_value_1: usize;
    let ret_value_2: usize;
    let s2_value: usize;
    unsafe {
        asm!(
        "li a0, 60",
        "li a1, 60",
        "li a2, 60",
        "li a3, 60",
        "li a4, 60",
        "li a5, 60",
        "li x16, 0x1",
        "li x17, 0x08475bd0",
        "ecall",
        out("x10") ret_value_1,
        out("x11") ret_value_2,
        out("x12") _,
        out("x13") _,
        out("x14") _,
        out("x15") _,
        out("x16") _,
        out("x17") _,
        out("x18") s2_value,
        );
    }

    ret_value_1 == 0xdeadbeef && ret_value_2 == 0xdeadbeef && s2_value != 0xdeadbeef
}
