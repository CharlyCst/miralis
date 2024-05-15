#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    // Setup some values                : firmware
    // Jump into OS function with mret  : firmware -> OS
    // Modify registers                 : OS
    // OS exception with ecall          : OS -> firmware
    // Check values                     : firmware

    let mut t6: usize;
    let mut sscratch: usize;
    let mut mstatus: usize;
    unsafe {
        let os: usize = _raw_os as usize;
        let trap: usize = _raw_trap_handler as usize;
        let mpp = 0b1 << 11; // MPP = S-mode

        asm!(
            "auipc t4, 0",
            "addi t4, t4, 24",
            "csrw mtvec, {mtvec}", // Write mtvec with trap handler
            "csrw mstatus, {mpp}", // Write MPP of mstatus to S-mode
            "csrw mepc, {os}",     // Write MEPC

            "mret",                // Jump to OS

            "csrr {sscratch}, sscratch ",
            "csrr {mstatus}, mstatus ",

            os = in(reg) os,
            mtvec = in(reg) trap,
            mpp = in(reg) mpp,
            sscratch = out(reg) sscratch,
            mstatus = out(reg) mstatus,
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

// ———————————————————————————————— Guest OS ———————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_os
_raw_os:
    li t6, 0x42        // Store a secret value into t6 before jumping to firmware
    csrw sscratch, t6  // Store a secret value into sscratch before jumping to firmware
    ecall
"#,
);

extern "C" {
    fn _raw_trap_handler();
    fn _raw_os();
}
