#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::usize;

use miralis_abi::{failure, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    test_continious_timer_interrupts();
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;
const MSIP_WIDTH: usize = 0x4;

// Set msip bit pending for other hart
fn set_msip(hart: usize, value: u32) {
    log::debug!("Set interrupt for hart {:}", hart);

    let msip_ptr = (CLINT_BASE + MSIP_WIDTH * hart) as *mut u32;
    unsafe {
        msip_ptr.write_volatile(value);
    }
}

#[allow(unreachable_code)]
fn test_continious_timer_interrupts() -> ! {
    let hart_id: usize;
    unsafe {
        asm!(
            "csrr {0}, mhartid",
            "csrw mtvec, {handler}",
            out(reg) hart_id,
            handler = in(reg) _raw_interrupt_trap_handler as usize,
        );
    }

    assert!(hart_id < 2, "Expected only 2 harts for this test");

    match hart_id {
        0 => {
            unsafe {
                asm!(
                    "csrs mstatus, {mstatus_mie}",  // Enable interrupts (MIE)
                    "csrs mie, {msie}",             // Enable software timer interrupt (MSIE)
                    "wfi",                          // Wait for other hart to send an interrupt
                    mstatus_mie = in(reg) 0x8,
                    msie = in(reg) 0x8,
                );
            }
            success()
        }
        1 => {
            set_msip(0, 1);
            loop {
                unsafe { asm!("nop") };
            }
        }
        _ => failure(),
    }
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    let mcause: usize;
    unsafe {
        asm!(
            "csrc mstatus, {mstatus_mie}", // Disable interrupts
            "csrr {0}, mcause",
            out(reg) mcause,
            mstatus_mie = in(reg) 0x8,
        );
    }

    if mcause == 0x8000000000000003 {
        set_msip(0, 0);
        unsafe {
            asm!("mret",);
        }
    } else {
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
