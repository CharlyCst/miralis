#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    log::debug!("Testing mie register");
    test_mie();
    log::debug!("Testing sie register");
    test_sie();
    log::debug!("Testing sie by mie register");
    test_sie_by_mie();
    log::debug!("Testing CLINT");
    unsafe {
        let handler = _raw_interrupt_trap_handler as usize;
        asm!(
            "csrw mtvec, {0}", // Write mtvec
            in(reg) handler,
        );
        test_timer_interrupts();
        success();
    }
}

// —————————————————————————————— mie and sie ——————————————————————————————— //

// Test mie with a simple read and write
fn test_mie() {
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mie, {0}",
            "csrr {1}, mie",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0x42);
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

// —————————————————————————————— M-mode internal interrupts ——————————————————————————————— //

// Enable global interrupts (MIE) and set MTIE bit in MIE register
fn enable_timer_interrupts() {
    // Set MIE bit (3rd bit) in mstatus register
    let mut tmp: usize;
    unsafe {
        asm!(
            "li {0}, 0x8",
            "csrs mstatus, {0}", // Enable interrupts (MIE)
            out(reg) tmp,
        );
    }
    assert_eq!(tmp, 0x8, "MIE bit set in mstatus");

    // Set MTIE bit (7th bit) in mie register
    unsafe {
        asm!(
            "li {0}, 0x80",
            "csrs mie, {0}", // Enable machine timer interrupt (MTIE)
            out(reg) tmp,
        );
    }
    assert_eq!(tmp, 0x80, "MTIE bit set in mie");
}

// Get the current mtime value
fn get_current_mtime() -> usize {
    let mtime_ptr = 0x200BFF8 as *const usize;
    unsafe { mtime_ptr.read_volatile() }
}

// Set mtimecmp value in the future
fn set_mtimecmp_future_value() {
    let current_mtime = get_current_mtime();
    let future_time = current_mtime + 10000;

    let mtimecmp_ptr = 0x2004000 as *mut usize; // TODO: add support for different harts
    unsafe {
        mtimecmp_ptr.write_volatile(future_time);
    }

    // Read back the value to verify
    let read_back = unsafe { mtimecmp_ptr.read_volatile() };
    assert_eq!(read_back, future_time, "mtimecmp set correctly");
}

// Wait for the timer interrupt
fn wait_for_timer_interrupt() {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

// Assert that MTIP flag is set
fn assert_timer_interrupt_flag() {
    let mip: usize;
    unsafe {
        asm!(
            "csrr {0}, mip",
            out(reg) mip,
        );
    }
    assert!(mip & 0x80 != 0, "MTIP flag set");
}

fn test_timer_interrupts() {
    enable_timer_interrupts();
    set_mtimecmp_future_value();
    wait_for_timer_interrupt();
    assert_timer_interrupt_flag();
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    // Test here
    let tmp: usize;
    unsafe {
        asm!(
            "li {0}, 0x80",
            "csrc mie, {0}",
            out(reg) tmp,
        );
    }
    // mcause = interrupt
    let bp_code: usize = 0x8000000000000007;
    let mut res: usize;
    unsafe {
        asm!(
            "csrr {0}, mcause",
            out(reg) res,
        );
    }

    read_test(res, bp_code);
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
    j {trap_handler} // Return imediately
"#,
    trap_handler = sym trap_handler,
);

extern "C" {
    fn _raw_interrupt_trap_handler();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
