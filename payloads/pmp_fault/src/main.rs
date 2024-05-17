#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::ptr;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    let mut mcause: usize;
    unsafe {
        let trap: usize = _raw_trap_handler as usize;

        asm!(
            "csrw mtvec, {mtvec}", // Write mtvec with trap handler

            mtvec = in(reg) trap,
        );
    }

    unsafe {
        let x: usize = 0x80002000;
        let y = x as *const usize;
        ptr::read_volatile(y);
    }

    unsafe {
        let trap: usize = _raw_trap_handler as usize;

        asm!(
            "csrr t6, mcause", // Write mtvec with trap handler

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
    fn _raw_os();
}
