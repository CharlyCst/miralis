#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::usize;

use miralis_abi::{failure, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::debug!("Testing CLINT");
    test_continious_timer_interrupts();
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;
static mut COUNTER: usize = 10;

// Get the current mtime value
fn get_current_mtime() -> usize {
    let mtime_ptr = (CLINT_BASE + MTIME_OFFSET) as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

// Set mtimecmp value in the future
fn set_mtimecmp_future_value(value: usize) {
    log::debug!("Updated timer");
    let current_mtime = get_current_mtime();
    let future_time = current_mtime.saturating_add(value);

    let mtimecmp_ptr = (CLINT_BASE + MTIMECMP_OFFSET) as *mut usize;
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }
}

#[allow(unreachable_code)]
fn test_continious_timer_interrupts() -> ! {
    // Setup a timer deadline
    set_mtimecmp_future_value(100000);

    // Configure trap handler and enable interrupts
    unsafe {
        asm!(
            "csrw mtvec, {handler}",       // Setup trap handler
            "csrs mstatus, {mstatus_mie}", // Enable interrupts (MIE)
            "csrs mie, {mtie}",            // Enable machine timer interrupt (MTIE)
            handler = in(reg) _raw_interrupt_trap_handler as usize,
            mstatus_mie = in(reg) 0x8,
            mtie = in(reg) 0x80,
        );
    }

    while unsafe { COUNTER } > 0 {
        unsafe { asm!("wfi") }; // If execution halts here, it's possible that r/o MTIP bit gets written
    }

    for i in 1..100 {
        unsafe { asm!("addi x0, x0, 1") };
    }

    success();
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    let mip: usize;
    let mcause: usize;
    unsafe {
        asm!(
            "csrc mstatus, {mstatus_mie}", // Disable interrupts (MIE)
            "csrr {0}, mip",
            "csrr {1}, mcause",
            out(reg) mip,
            out(reg) mcause,
            mstatus_mie = in(reg) 0x8,
        );
    }

    if mcause == 0x8000000000000007 {
        log::debug!("Trapped on interrupt, mip {:b}", mip);

        if unsafe { COUNTER } <= 0 {
            failure()
        }; // Shouldn't trap if the interrupt pending bit was cleared
        unsafe { COUNTER -= 1 };
        log::debug!("counter {:?}", unsafe { COUNTER });

        // Do not actually clear the interrupt first time: expect to trap again
        if unsafe { COUNTER == 0 } {
            set_mtimecmp_future_value(usize::MAX);
            log::debug!("Now should clear");
        };

        unsafe {
            asm!(
                // "csrr {0}, mepc",
                // "addi {0}, {0}, 4",
                // "csrw mepc, {0}",
                "csrc mip, {mip_mtie}",         // Try clearing mtip directly - shouldn't work
                "csrs mstatus, {mstatus_mie}",  // Enable interrupts (MIE)
                "mret",
                // out(reg) _,
                mstatus_mie = in(reg) 0x8,
                mip_mtie = in(reg) 0x80,
            );
            // log::debug!("ret to {:x}", ret);
        }
    } else {
        log::debug!("Not a timer exception!");
        // failure();
        unsafe {
            asm!(
                "csrr {0}, mepc",
                "addi {0}, {0}, 4",
                "csrw mepc, {0}",
                "csrs mstatus, {mstatus_mie}", // Enable interrupts (MIE)
                "mret",
                out(reg) _,
                mstatus_mie = in(reg) 0x8,
            );
            // log::debug!("ret to {:x}", ret);
        }
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
