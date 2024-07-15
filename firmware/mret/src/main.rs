#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    let mut mstatus: usize;
    let mut t6: usize;
    unsafe {
        let handler = _raw_breakpoint_trap_handler as usize;
        // Let's rise an exception breakpoint directly
        asm!(
            "csrw mtvec, {0}",   // Write mtvec
            "ebreak",            // Cause an exception, we should return right away!
            "csrr {1}, mstatus", // Read mstatus
            in(reg) handler,
            out(reg) mstatus,
            out("t6") t6,        // The handler writes a secret value in t6
        );
    }

    let mpp = (mstatus >> 11) & 0b11;
    let mpie = (mstatus >> 7) & 0b1;
    let mprv = (mstatus >> 17) & 0b1;

    assert_eq!(mpp, 0, "Invalid MPP: {}, expected 0", mpp);
    assert_eq!(mpie, 1, "Invalid MPIE: {}, expected 1", mpie);
    assert_eq!(mprv, 0, "Invalid MPRV: {}, expected 0", mprv);
    assert_eq!(
        t6, 0x42,
        "Trap handler did not properly update the value in t6"
    );

    success();
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_breakpoint_trap_handler
_raw_breakpoint_trap_handler:
    csrr t6, mepc  // Read EPC
    addi t6, t6, 4 // Increment return pointer
    csrw mepc, t6  // Write it back
    li t6, 0x42    // And store a secret value in t6 before returning
    mret
"#,
);

extern "C" {
    fn _raw_breakpoint_trap_handler();
}
