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
    let mut sscratch: usize;
    let mut mstatus: usize;
    unsafe {
        let os: usize = _raw_os as usize;
        let trap: usize = _raw_trap_handler as usize;

        let value = 0b1 << 11;

        asm!(
            "auipc t4, 0",
            "addi t4, t4, 24",
            "csrw mtvec, {1}",   // Write mtvec with trap handler
            "csrw mstatus, {2}", // Write MPP of mstatus to S-mode
            "csrw mepc, {0}",   // Write MEPC

            "mret",            // Jump to OS

            "csrr {3}, sscratch ",
            "csrr {4}, mstatus ",

            in(reg) os,
            in(reg) trap,
            in(reg) value,
            out(reg) sscratch,
            out(reg) mstatus,
            out("t6") t6,
        );
    }

    assert_eq!(t6, 0x42, "OS did not properly update the value in t6");
    assert_eq!(
        sscratch, 0x42,
        "OS did not properly update the value in sscratch"
    );
    assert_eq!(
        (mstatus >> 11) & 0b11,
        0b01,
        "Check that mstatus MPP is still in S-mode"
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
    csrw sscratch, t6     // Store a secret value into sscratch before jumping to firmware
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
