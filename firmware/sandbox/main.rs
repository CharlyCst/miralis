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
    let x: usize = 0x80002000;
    // Check access fault with default memory privilege
    unsafe {
        // Setup trap handler
        asm!(
            "csrw mtvec, {handler}",
            handler = in(reg) _raw_trap_handler as usize,
        );

        // Try to read (Load) an address that should be protected by Miralis
        ptr::read_volatile(x as *const usize);

        // Check what error we encountered
        asm!(
            "csrr t6, mcause",
            out("t6") mcause,
        );
    }

    assert_eq!(mcause, 0x5, "exception was not load access fault");

    // Check access fault with memory privilege modified
    // It's not a complete test of accesses with MPRV bit active,
    // Since SATP are not installed, no address translation is happenning in fact
    unsafe {
        let mpp: i32 = 0b1 << 11; // MPP = S-mode
        asm!(
            "li {0}, 1",
            "slli {0}, {0}, 17",
            "csrs mstatus, {0}",    // Set MPRV bit to 1
            "csrw mstatus, {mpp}",  // Set MPP to S-mode
            "sd x0, 0({x})",        // Try to store something at a protected address
            "csrr t6, mcause",
            out(reg) _,
            x = in(reg) x,
            mpp = in(reg) mpp,
            out("t6") mcause,
        );
    }

    assert_eq!(mcause, 0x7, "exception was not store access fault");

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

unsafe extern "C" {
    fn _raw_trap_handler();
}
