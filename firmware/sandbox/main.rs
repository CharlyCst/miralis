//! Sandboxing test
//!
//! This test firmware checks that Miralis is properly protected, or in other words that the
//! firmware is properly sandboxed.

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::ptr;

use miralis_abi::{setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    let mut mcause: usize;
    unsafe {
        // Setup trap handler
        asm!(
            "csrw mtvec, {handler}",
            handler = in(reg) _raw_trap_handler as usize,
        );

        // Try to read an address that should be protected by Miralis
        let x: usize = 0x80002000;
        ptr::read_volatile(x as *const usize);

        // Check what error we encountered
        asm!(
            "csrr t6, mcause",
            out("t6") mcause,
        );
    }

    assert_eq!(mcause, 0x5, "exception was not access fault");

    success();
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
