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

    let os: usize = 0x80400000usize;
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
    // Skip instruction (pc += 4)
    csrrw x5, mepc, x5
    addi x5, x5, 4
    csrrw x5, mepc, x5

    // If ecall test --> jump to it
    csrr t0, mcause
    li   t1, 9
    beq  t0, t1, test_ecall

    // Set x3 to 0 - for test_same_registers_after_trap
    li t6, 0

    // If mcause is 7 or 2, it might be triggered by the instruction sd t5, 0(t6) | csrr t0, mcause
    // Therefore we don't want to load it a second time
    csrr t0, mcause
    li   t1, 7
    beq  t0, t1, done
    csrr t0, mcause
    li   t1, 2
    beq  t0, t1, done

    // Make sure we can't change the payload memory
    li t6, 0x80400000
    li t5, 60
    sd t5, 0(t6)

test_ecall:
    // Test Ecall
    csrr t0, mcause
    li   t1, 2
    bne  t0, t1, done

    // Make sure Ecall forwarding rule works by checking if all input registers are equal to 60
    li   t2, 60
    bne  a0, t2, infinite_loop
    //bne  a1, t2, infinite_loop
    //bne  a2, t2, infinite_loop
    //bne  a3, t2, infinite_loop
    //bne  a4, t2, infinite_loop
    //bne  a5, t2, infinite_loop
    //bne  a6, t2, infinite_loop
    //bne  a7, t2, infinite_loop
    // Set return registers to 61
    // li a0, 61
    // li a1, 61
    j done

// TODO: make a failure ecall here
infinite_loop:
    j infinite_loop
done:
    // Return back to miralis
    mret
#"
);

extern "C" {
    fn _raw_trap_handler();
}
