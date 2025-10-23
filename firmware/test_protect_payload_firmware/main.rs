//! Test protect payload policy
//!
//! This payload serve as test firmware for the protect payload policy. It must be used with the payload test_protect_payload_payload only.
//! These two components together make sure we enforce the protect payload policy correctly.

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use miralis_abi::{failure, setup_binary};
use miralis_config::TARGET_PAYLOAD_ADDRESS;

setup_binary!(main);

fn main() -> ! {
    install_trap_handler();

    let os: usize = TARGET_PAYLOAD_ADDRESS;
    let mpp = 0b1 << 11; // MPP = S-mode

    unsafe {
        asm!(
        "li t4, 0xfffffffff",
        "csrw pmpcfg0, 0xf",   // XRW TOR
        "csrw pmpaddr0, t4",   // All memory
        "csrw mstatus, {mpp}", // Write MPP of mstatus to S-mode
        "csrw mepc, {os}",     // Write MEPC
        "li a0, 1",

        "mret",                // Jump to OS


        os = in(reg) os,
        mpp = in(reg) mpp,
        );
    }

    // Unreachable code
    failure();
}

fn install_trap_handler() {
    unsafe {
        asm!("csrw mtvec, {mtvec}", mtvec = in(reg) _raw_trap_handler as usize);
    }
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r"
.text
.align 4
.global _raw_trap_handler
_raw_trap_handler:
    // Advance PC by 4 (skip instruction)
    csrrw x5, mepc, x5
    addi  x5, x5, 4
    csrrw x5, mepc, x5

    // Verify if all input registers are equal to 60
    li    t2, 60
    bne   a0, t2, infinite_loop
    bne   a1, t2, infinite_loop
    bne   a2, t2, infinite_loop
    bne   a3, t2, infinite_loop
    bne   a4, t2, infinite_loop
    bne   a5, t2, infinite_loop

    // Set return values for successful ecall
    li    a0, 0xdeadbeef
    li    a1, 0xdeadbeef
    j     done

infinite_loop:
    // Infinite loop in case of failure
    wfi
    j     infinite_loop

done:
    // Make sure we can't read this 0xdeadbeef in the payload
    li    s2, 0xdeadbeef

    // Return to Miralis
    mret
"
);

unsafe extern "C" {
    fn _raw_trap_handler();
}
