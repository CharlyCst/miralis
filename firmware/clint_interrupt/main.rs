#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::usize;

use miralis_abi::{failure, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Setup a timer deadline
    set_mtimecmp_future_value(10000);

    // Configure trap handler and enable interrupts
    unsafe {
        asm!(
            "csrw mtvec, {handler}",       // Setup trap handler
            "csrs mie, {mtie}",            // Enable machine timer interrupt (MTIE)
            handler = in(reg) _raw_interrupt_trap_handler as usize,
            mtie = in(reg) 0x80,
        );
    }

    for _ in 1..REPETITIONS {
        unsafe {
            asm!(
                "csrs mstatus, {mstatus_mie}",  // Enable interrupts (MIE)
                "nop",
                mstatus_mie = in(reg) 0x8,
            )
        };
    }
    unsafe {
        asm!("ebreak");
    }
    failure()
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;
const REPETITIONS: usize = 10;

static mut INTERRUPT_COUNTER: usize = 0;

// Get the current mtime value
fn get_current_mtime() -> usize {
    let mtime_ptr = (CLINT_BASE + MTIME_OFFSET) as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

// Set mtimecmp value in the future
fn set_mtimecmp_future_value(value: usize) {
    let current_mtime = get_current_mtime();
    let future_time = current_mtime.saturating_add(value);

    let mtimecmp_ptr = (CLINT_BASE + MTIMECMP_OFFSET) as *mut usize;
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    let mip: usize;
    let mcause: usize;
    let mepc: usize;

    unsafe {
        asm!(
            "csrc mstatus, {mstatus_mie}", // Disable interrupts (MIE)
            "csrr {0}, mip",
            "csrr {1}, mcause",
            "csrr {2}, mepc",
            out(reg) mip,
            out(reg) mcause,
            out(reg) mepc,
            mstatus_mie = in(reg) 0x8,
        );
    }

    if mcause == 0x8000000000000007 {
        log::trace!("Trapped on interrupt, mip {:b}, at {:x}", mip, mepc);

        if unsafe { INTERRUPT_COUNTER } > REPETITIONS {
            failure()
        }; // Shouldn't trap if the interrupt pending bit was cleared

        log::trace!("Counter {:?}", unsafe { INTERRUPT_COUNTER });

        // Do not actually clear the interrupt first time: expect to trap again
        if unsafe { INTERRUPT_COUNTER } == REPETITIONS {
            set_mtimecmp_future_value(usize::MAX);
            log::trace!("Now timer should clear");
            unsafe {
                asm!("mret",);
            }
        };

        unsafe {
            INTERRUPT_COUNTER += 1;
            // Try clearing mtip directly - shouldn't work
            asm!(
                "csrc mip, {mip_mtie}",
                "mret",
                mip_mtie = in(reg) 0x80,
            );
        }
    } else if mcause == 3 {
        if unsafe { INTERRUPT_COUNTER } == REPETITIONS {
            success()
        } else {
            log::warn!(
                "Expected to get interrupt {} times, got {}",
                REPETITIONS,
                unsafe { INTERRUPT_COUNTER }
            );
            failure()
        }
    } else {
        log::debug!("Not a timer exception! {}", mcause);
        failure();
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
