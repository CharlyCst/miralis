//! Test protect payload policy
//!
//! This payload serve as test firmware for the protect payload policy. It must be used with the payload test_protect_payload_payload only.
//! These two components together make sure we enforce the protect payload policy correctly.

#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{failure, setup_binary};

setup_binary!(main);

fn main() -> ! {
    let os: usize = 0x80400000 as usize;
    let mpp = 0b1 << 11; // MPP = S-mode

    unsafe {
        asm!(
        "li t4, 0xfffffffff",
        "csrw pmpcfg0, 0xf",   // XRW TOR
        "csrw pmpaddr0, t4",   // All memory
        "csrw mstatus, {mpp}", // Write MPP of mstatus to S-mode
        "csrw mepc, {os}",     // Write MEPC
        "li a0, 1",

        "mret",                // Jump to OS


        os = in(reg) os,
        mpp = in(reg) mpp,
        );
    }

    // Unreachable code
    failure();
}
