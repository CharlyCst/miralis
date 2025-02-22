#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use miralis_abi::{failure, setup_binary};

setup_binary!(main);

fn main() -> ! {
    // Setup some values                : firmware
    // Jump into OS function with mret  : firmware -> OS
    // Ecall from S-Mode                : OS -> Miralis
    // ecall intercepted by Miralis     : exit

    let os: usize = _raw_os as usize;
    let trap: usize = _raw_trap_handler as usize;
    let mpp: i32 = 0b1 << 11; // MPP = S-mode

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

            os = in(reg) os,
            mtvec = in(reg) trap,
            mpp = in(reg) mpp,
        );
    }
    failure()
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
    li a6, 1           // Miralis ABI FID: Exit with success
    li a7, 0x08475bcd  // Miralis ABI EID
    ecall
"#,
);

extern "C" {
    fn _raw_trap_handler();
    fn _raw_os();
}
