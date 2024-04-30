#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {

    // Setup some values                : firmware
    // Jump into OS function with mret  : firmware -> OS
    // Modify registers                 : OS
    // OS exception with ebreak/ecall   : OS -> firmware
    // Check values                     : firmware

    let mut mstatus: usize;
    let mut t6: usize;
    unsafe {
        let os = _raw_os as usize;
        let trap: usize = _raw_trap_handler as usize;
        // Let's rise an exception breakpoint directly

        asm!(
            "csrw mtvec, {0}",   // Write mtvec with trap handler 
            "crsw mstatus, 0x42", // Write MPP of mstatus to S-mode 
            "csrw mepc, {1}",   // Write MEPC 
            "mret",            // Jump to OS 

            "csrr {2}, mstatus", // Read mstatus 
            in(reg) os,
            in(reg) trap,
            out(reg) mstatus,
            out("t6") t6,        // The OS writes a secret value in t6 
        );
    }
    
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
.global _raw_trap_handler
_raw_trap_handler:
    csrr t6, mepc  // Read EPC
    addi t6, t6, 4 // Increment return pointer
    csrw mepc, t6  // Write it back
    li t6, 0x42    // And store a secret value in t6 before returning
    mret
"#,
);

global_asm!(
    r#"
.text
.align 4
.global _raw_os
_raw_os:
    csrr t6, mepc  // Read EPC
    addi t6, t6, 4 // Increment return pointer
    csrw mepc, t6  // Write it back
    li t6, 0x42    // And store a secret value in t6 before returning
    ebreak
"#,
);

extern "C" {
    fn _raw_trap_handler();
    fn _raw_os();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
