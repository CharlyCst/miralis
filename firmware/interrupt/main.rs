#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use miralis_abi::{failure, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::debug!("Testing mie register");
    test_mie();
    log::debug!("Testing sie register");
    test_sie();
    log::debug!("Testing sie by mie register");
    test_sie_by_mie();
    log::debug!("Testing CLINT");
    test_timer_interrupts();
}

// —————————————————————————————— mie and sie ——————————————————————————————— //

// Test mie with a simple read and write
fn test_mie() {
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0xbbb",
            "csrw mie, {0}",
            "csrr {1}, mie",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(
        res, 0xaaa,
        "Mie need to be writable, and only on writable bits"
    );
}

// Test sie: it should be masked by S-mode bit only
fn test_sie() {
    let sie: usize;
    let mie: usize;
    let value = 0x3ff;
    let masked_value = value & 0x222;

    unsafe {
        asm!(
            "csrw sie, {value}",
            "csrr {sie}, sie",
            "csrr {mie}, mie",
            sie = out(reg) sie,
            mie = out(reg) mie,
            value = in(reg) value,
        );
    }

    assert_eq!(
        sie, masked_value,
        "sie is correctly set to the masked value"
    );
    assert_eq!(mie & 0x222, masked_value, "mie S bits need to be set");
}

// Test sie: writting to mie must be
fn test_sie_by_mie() {
    let res: usize;
    let value = 0x3ff;
    let masked_value = value & 0x222;
    unsafe {
        asm!(
            "csrw mie, {value}",
            "csrr {0}, sie",
            out(reg) res,
            value = in(reg) value,
        );
    }

    assert_eq!(res, masked_value);
}

// ———————————————————————————— Timer Interrupt ————————————————————————————— //

const CLINT_BASE: usize = 0x2000000;
const MTIME_OFFSET: usize = 0xBFF8;
const MTIMECMP_OFFSET: usize = 0x4000;

// Get the current mtime value
fn get_current_mtime() -> usize {
    let mtime_ptr = (CLINT_BASE + MTIME_OFFSET) as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

// Set mtimecmp value in the future
fn set_mtimecmp_future_value() {
    let current_mtime = get_current_mtime();
    let future_time = current_mtime + 10000;

    let mtimecmp_ptr = (CLINT_BASE + MTIMECMP_OFFSET) as *mut usize; // TODO: add support for different harts
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }

    // Read back the value to verify
    let read_back = unsafe { mtimecmp_ptr.read_volatile() };
    assert_eq!(read_back, future_time, "mtimecmp set correctly");
}

#[allow(unreachable_code)]
fn test_timer_interrupts() -> ! {
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

    // Setup a timer deadline
    set_mtimecmp_future_value();

    // Wait for an interrupt
    loop {
        unsafe { asm!("wfi") };
    }

    // The trap handler should exit, if we reach that point the handler did not do its job
    failure();
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    // Check the interrupt cause
    let expected_mcause: usize = 0x8000000000000007;
    let mut mcause: usize;
    unsafe {
        asm!(
            "csrr {0}, mcause",
            out(reg) mcause,
        );
    }
    assert_eq!(
        mcause, expected_mcause,
        "Expected to receive a timer interrupt, got something else"
    );

    let mip: usize;
    unsafe {
        asm!(
            "csrr {0}, mip",
            out(reg) mip,
        );
    }
    assert!(mip & 0x80 != 0, "MTIP flag set");

    log::debug!("Done!");
    success();
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
