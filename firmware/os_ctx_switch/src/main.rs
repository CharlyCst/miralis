#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    // Setup some values                : firmware
    // Jump into OS function with mret  : firmware -> OS
    // Modify registers                 : OS
    // OS exception with ecall          : OS -> firmware
    // Check values                     : firmware

    let mut t6: usize;
    let mut sscratch: usize;
    let mut mstatus: usize;
    let mut mepc: usize;

    let os: usize = _raw_os as usize;
    let trap: usize = _raw_trap_handler as usize;
    let mpp = 0b1 << 11; // MPP = S-mode

    unsafe {
        asm!(
            "li t4, 0xfffffffff",
            "csrw pmpcfg0, 0xf",   // XRW TOR
            "csrw pmpaddr0, t4",   // All memory
            "auipc t4, 0",
            "addi t4, t4, 24",
            "csrw mtvec, {mtvec}", // Write mtvec with trap handler
            "csrw mstatus, {mpp}", // Write MPP of mstatus to S-mode
            "csrw mepc, {os}",     // Write MEPC

            "mret",                // Jump to OS

            "csrr {sscratch}, sscratch ",
            "csrr {mstatus}, mstatus ",
            "csrr {mepc}, mepc",

            os = in(reg) os,
            mtvec = in(reg) trap,
            mpp = in(reg) mpp,
            sscratch = out(reg) sscratch,
            mstatus = out(reg) mstatus,
            mepc = out(reg) mepc,
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

    assert_eq!(
        mepc,
        os + 4 * 2, // mepc must be set to where the trap occured (ecall)
        "Check that mepc has good value"
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
