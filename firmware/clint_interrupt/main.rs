#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::usize;

use miralis_abi::{setup_binary, success};
use test_helpers::clint::{self, set_mtimecmp_deadline};

setup_binary!(main);

/// This test verifies the general functionality of MTI (Machine Timer Interrupt) virtualization, including:
/// 1. The timer interrupt is delivered within a reasonable time when the timer is set and interrupts are enabled.
/// 2. Firmware cannot directly clear the M-mode timer interrupt (MTI) by writing to `vMIP.MTIP`.
/// 3. Writing to the appropriate memory-mapped register correctly clears the interrupt.
/// 4. If the virtual interrupt is not cleared, the firmware will trap on each execution cycle.
fn main() -> ! {
    // Setup a timer deadline
    clint::set_mtimecmp_deadline(0, 0);

    // Configure trap handler and enable interrupts
    unsafe {
        asm!(
            "csrw mtvec, {handler}",       // Setup trap handler
            "csrs mie, {mtie}",            // Enable machine timer interrupt (MTIE)
            handler = in(reg) _raw_interrupt_trap_handler as usize,
            mtie = in(reg) 0x80,
        );

        // Enable interrupts in `mstatus`
        enable_interrupts();
    }

    panic!("Expected a timer interrupt, but did not trap");
}

/// Enables interrupts by setting `mstatus.MIE` to 1.
unsafe fn enable_interrupts() {
    asm!(
        "csrs mstatus, {mstatus_mie}",  // Enable global interrupts (MIE), expect to trap immediately
        mstatus_mie = in(reg) 0x8,
    )
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

static INTERRUPT_COUNTER: AtomicUsize = AtomicUsize::new(0);
const REPETITIONS: usize = 10;

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    let mip: usize;
    let mcause: usize;
    let mepc: usize;

    unsafe {
        asm!(
            "csrr {0}, mip",
            "csrr {1}, mcause",
            "csrr {2}, mepc",
            out(reg) mip,
            out(reg) mcause,
            out(reg) mepc,
        );

        log::trace!("Trapped on interrupt, mip {:b}, at {:x}", mip, mepc);
    }

    if mcause == 0x8000000000000007 {
        handle_timer_interrupt();
    } else {
        panic!("Not a timer exception! {}", mcause);
    }
}

fn handle_timer_interrupt() {
    let count = INTERRUPT_COUNTER.load(Ordering::SeqCst);

    if count > REPETITIONS {
        // Shouldn't trap if the interrupt pending bit was cleared
        panic!("Failed to clear timer interrupts")
    } else if count < REPETITIONS {
        // Increment the counter
        INTERRUPT_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Now try to clear the interrupt directly (which shouldn't work) and enable the interrupts
        // again. We expect a new trap, thus re-entering this function.
        unsafe {
            // Try clearing mtip directly - shouldn't work
            asm!(
                "csrc mip, {mip_mtie}",
                mip_mtie = in(reg) 0x80,
            );

            enable_interrupts();
        }

        panic!("Expected another timer interrupt, but got nothing");
    } else {
        // We got all the timers we expected!
        set_mtimecmp_deadline(usize::MAX, 0);
        log::trace!("Now timer should clear, re-enabling interrupts");
        unsafe { enable_interrupts() };

        // If we reach here it means the timer interrupt was cleared properly
        success();
    };
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
