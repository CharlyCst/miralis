#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{failure, setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    // Setup some values                : firmware
    // Jump into OS function with mret  : firmware -> OS
    // Modify registers                 : OS
    // OS exception with ebreak/ecall   : OS -> firmware
    // Check values                     : firmware

    let mut t6: usize;
    let mut scause: usize;
    unsafe {
        let os: usize = _raw_os as usize;
        let trap: usize = _raw_trap_handler as usize;
        let fake: usize = _raw_fake_trap_handler as usize;
        // Let's rise an exception breakpoint directly

        let value = 0b1 << 11;

        asm!(
            "csrw mtvec, {3}",   // Write mtvec with fake trap handler

            "ebreak",   //Jump to fake trap handler

            "addi t4, t4, 20",
            "csrw mtvec, {1}",   // Write mtvec with trap handler
            "csrw mstatus, {2}", // Write MPP of mstatus to S-mode
            "csrw mepc, {0}",   // Write MEPC

            "mret",            // Jump to OS

            "csrr {4},scause ",

            in(reg) os,
            in(reg) trap,
            in(reg) value,
            in(reg) fake,
            out(reg) scause,
            out("t6") t6,
        );
    }

    assert_eq!(t6, 0x42, "OS did not properly update the value in t6");
    assert_eq!(scause, 0x42, "OS did not properly update the value in t6");

    success();
}

global_asm!(
    r#"
.text
.align 4
.global _raw_fake_trap_handler
_raw_fake_trap_handler:
    csrr t4, mepc  // Read EPC
    addi t4, t4, 4 // Increment return pointer
    jr t4
"#,
);

// —————————————————————————————— Trap Handler —————————————————————————————— //
global_asm!(
    r#"
.text
.align 4
.global _raw_trap_handler
_raw_trap_handler:
    jr t4
"#,
);

// —————————————————————————————— Guest OS —————————————————————————————— //
global_asm!(
    r#"
.text
.align 4
.global _raw_os
_raw_os:
    li t6, 0x42    // Store a secret value into t6 before jumping to firmware
    csrw scause, t6     
    ebreak  
"#,
);

extern "C" {
    fn _raw_trap_handler();
    fn _raw_os();
    fn _raw_fake_trap_handler();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
