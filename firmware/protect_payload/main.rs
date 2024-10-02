//! Protect payload test
//!
//! This test firmware checks that Miralis is properly protected, or in other words that the
//! firmware is properly sandboxed.

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::ptr;

use miralis_abi::{lock_payload, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Lock the payload from the firmware
    lock_payload();

    // Make sure r/w to Payload leads to an error
    try_rw_protected_payload();

    // We can exit with success
    success();
}

fn try_rw_protected_payload() {
    unsafe {
        asm!("csrw mcause, zero");
    }

    let mut mcause: usize;
    let address_in_payload: usize = 0x80400000;
    // Check access fault with default memory privilege
    unsafe {
        // Try to read (Load) an address that should be protected by Miralis
        ptr::read_volatile(address_in_payload as *const usize);

        // Check what error we encountered
        asm!(
        "csrr t6, mcause",
        out("t6") mcause,
        );
    }

    assert_eq!(
        mcause, 0x5,
        "exception was not load access fault while loading from payload"
    );
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_trap_handler
_raw_trap_handler:
    csrr t6, mepc  // Read EPC
    addi t6, t6, 4 // Increment return pointer
    csrw mepc, t6  // Write it back
    mret
"#,
);

extern "C" {
    fn _raw_trap_handler();
}
