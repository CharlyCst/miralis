#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

use mirage_abi::{failure, success};

global_asm!(
    r#"
.text
.align 4
.global _start
_start:
    j {entry}
"#,
    entry = sym entry,
);

extern "C" fn entry() -> ! {
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

    // MPP = 0
    read_test((mstatus >> 11) & 0b11, 0);
    // MPIE = 1
    read_test((mstatus >> 7) & 0b1, 1);
    // MPRV = 0
    read_test((mstatus >> 17) & 0b1, 0);

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

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}

// ————————————————————————————— Panic Handler —————————————————————————————— //

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
