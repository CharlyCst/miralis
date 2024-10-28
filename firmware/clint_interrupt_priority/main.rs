#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::usize;

use miralis_abi::{failure, setup_binary, success};

setup_binary!(main);

/// This test verifies that the priority of clearing virtualized Machine Software Interrupt (MSI)
/// and Machine Timer Interrupt (MTI) is correct, with MSI taking precedence over MTI.
/// It also ensures that both interrupts can be properly cleared by writing to their respective
/// memory-mapped registers.
fn main() -> ! {
    // Set up two interrupts simultaniously
    set_msip(0, 1);
    set_mtimecmp_future_value(0);

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

const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;
const MSIP_WIDTH: usize = 0x4;

// Get the current mtime value
fn get_current_mtime() -> usize {
    let mtime_ptr = (CLINT_BASE + MTIME_OFFSET) as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

// Set mtimecmp value in the future
fn set_mtimecmp_future_value(value: usize) {
    log::trace!("Updated timer");
    let current_mtime = get_current_mtime();
    let future_time = current_mtime.saturating_add(value);

    let mtimecmp_ptr = (CLINT_BASE + MTIMECMP_OFFSET) as *mut usize;
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }
}

// Set msip bit pending for other hart
fn set_msip(hart: usize, value: u32) {
    log::trace!("Set interrupt for hart {:}", hart);

    let msip_ptr = (CLINT_BASE + MSIP_WIDTH * hart) as *mut u32;
    unsafe {
        msip_ptr.write_volatile(value);
    }
}

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
        set_msip(0, 0);
    } else if mcause == 0x8000000000000007 {
        log::trace!("Cleared MTIP");
        set_mtimecmp_future_value(usize::MAX);
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
