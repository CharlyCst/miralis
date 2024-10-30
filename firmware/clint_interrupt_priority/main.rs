#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::usize;

use miralis_abi::{failure, setup_binary, success};
use test_helpers::clint;

setup_binary!(main);

/// This test verifies that the priority of clearing virtualized Machine Software Interrupt (MSI)
/// and Machine Timer Interrupt (MTI) is correct, with MSI taking precedence over MTI.
/// It also ensures that both interrupts can be properly cleared by writing to their respective
/// memory-mapped registers.
fn main() -> ! {
    // Set up two interrupts simultaniously
    clint::send_msi(0);
    clint::set_mtimecmp_deadline(0, 0);

    // MTIP is not guaranteed to be reflected instantly, so wait for a bit
    for _ in 1..10000 {
        core::hint::spin_loop();
    }

    // Configure trap handler and enable interrupts
    unsafe {
        asm!(
            "csrw mtvec, {handler}",        // Setup trap handler
            "csrs mstatus, {mstatus_mie}",  // Enable interrupts
            "csrs mie, {mie}",              // Enable MTIE and MSIE
            handler = in(reg) _raw_interrupt_trap_handler as usize,
            mie = in(reg) 0x88,
            mstatus_mie = in(reg) 0x8,
        );
    }

    // Expect to trap on both interrupts one by one
    for _ in 1..10000 {
        core::hint::spin_loop();
    }

    let mip: usize;
    unsafe {
        asm!(
            "csrr {mip}, mip",
            mip = out(reg) mip,
        );
    }

    if mip == 0 {
        success();
    } else {
        panic!(
            "There should be no more pending interrupts, but mip is 0x{:x}",
            mip
        );
    }
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    let mip: usize;
    let mcause: usize;
    unsafe {
        asm!(
            "csrr {0}, mip",
            "csrr {1}, mcause",
            out(reg) mip,
            out(reg) mcause,
        );
    }
    log::trace!("MIP: {:b}", mip);

    let msip = (mip >> 3) & 1;
    let mtip = (mip >> 7) & 1;

    if mtip < msip {
        log::warn!("MTIP was cleared before MSIP!");
        failure();
    }

    if mcause == 0x8000000000000003 {
        log::trace!("Cleared MSIP");
        clint::clear_msi(0);
    } else if mcause == 0x8000000000000007 {
        log::trace!("Cleared MTIP");
        clint::set_mtimecmp_deadline(usize::MAX, 0);
    } else {
        log::warn!("Received non-interrupt trap, {:x}", mcause);
        failure();
    }

    // WARNING:
    // We are returning without resetting the registers to their previous state here, this might
    // cause undefined behavior when jumping back to the function that trapped.
    // We should create a proper utility library that provides trap handler helpers to ease testing
    // interrupts.
    unsafe {
        asm!("mret");
    }
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_interrupt_trap_handler
_raw_interrupt_trap_handler:
    j {trap_handler} // Jump immediately into the Rust trap handler
"#,
    trap_handler = sym trap_handler,
);

extern "C" {
    fn _raw_interrupt_trap_handler();
}
