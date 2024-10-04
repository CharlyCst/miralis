//! Test protect payload policy
//!
//! This payload serve as test firmware for the protect payload policy. It must be used with the payload test_protect_payload_payload only.
//! These two components together make sure we enforce the protect payload policy correctly.

#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use miralis_abi::{failure, setup_binary};

setup_binary!(main);

fn main() -> ! {
    install_trap_handler();

    let os: usize = 0x80400000;
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

    // Check mcause for ecall (mcause = 9)
    csrr  t0, mcause
    li    t1, 9
    beq   t0, t1, test_ecall

    // Set t6 to 0 for test_same_registers_after_trap
    li    t6, 0

    // Skip redundant mcause checks if already handled (mcause = 7 or 2)
    csrr  t0, mcause
    li    t1, 7
    beq   t0, t1, done
    li    t1, 2
    beq   t0, t1, done

    // Ensure payload memory cannot be altered
    li    t6, 0x80400000
    li    t5, 60
    sd    t5, 0(t6)

test_ecall:
    // Handle ecall (mcause = 9)
    csrr  t0, mcause
    li    t1, 9
    bne   t0, t1, done

    // Verify if all input registers are equal to 60
    li    t2, 60
    bne   a0, t2, infinite_loop
    bne   a1, t2, infinite_loop
    bne   a2, t2, infinite_loop
    bne   a3, t2, infinite_loop
    bne   a4, t2, infinite_loop
    bne   a5, t2, infinite_loop

    // Set return values for successful ecall
    li    a0, 61
    li    a1, 62
    j     done

infinite_loop:
    // Infinite loop in case of failure
    j     infinite_loop

done:
    // Write random values into registers for testing state integrity
    li    t0, 0x12345678
    li    t1, 0x9abcdef0
    li    t2, 0xdeadbeef
    li    t3, 0xc0ffee00
    li    t4, 0x00001234
    li    t5, 0x56789abc
    li    t6, 0x2468ace0
    li    s0, 0x11111111
    li    s1, 0x22222222
    li    a2, 0x33333333
    li    a3, 0x44444444
    li    a4, 0x55555555
    li    a5, 0x66666666
    li    a6, 0x77777777
    li    a7, 0x88888888
    li    s2, 0x99999999
    li    s3, 0xaaaaaaaa
    li    s4, 0xbbbbbbbb
    li    s5, 0xcccccccc
    li    s6, 0xdddddddd
    li    s7, 0xeeeeeeee
    li    s8, 0xffffffff
    li    t3, 0xabcdef01
    li    t4, 0x23456789
    li    t5, 0x98765432
    li    t6, 0x10fedcba
    li    gp, 0x00000000
    li    sp, 0x11110000
    li    ra, 0x22220000

    // Return to Miralis
    mret
"
);

extern "C" {
    fn _raw_trap_handler();
}
