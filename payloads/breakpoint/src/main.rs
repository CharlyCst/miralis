#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    unsafe {
        let handler = _raw_breakpoint_trap_handler as usize;
        // Let's rise an exception breakpoint directly
        asm!(
            "csrw mtvec, {0}", // Write mtvec
            "ebreak",          // Cause an exception
            in(reg) handler,
        );
    }

    panic!();
}

/// This function should be called from the raw trap handler
extern "C" fn trap_handler() {
    // Test here

    // mcause = breakpoint
    let bp_code: usize = 0x3;
    let mut res: usize;
    unsafe {
        asm!(
            "csrr {0}, mcause",
            out(reg) res,
        );
    }

    read_test(res, bp_code);

    //mstatus MPP = M-mode
    let mpp_code: usize = 0x3;
    unsafe {
        asm!(
            "csrr {0}, mstatus",
            out(reg) res,
        );
    }

    read_test((res >> 11) & 0b11, mpp_code);
    success();
}

// —————————————————————————————— Trap Handler —————————————————————————————— //

global_asm!(
    r#"
.text
.align 4
.global _raw_breakpoint_trap_handler
_raw_breakpoint_trap_handler:
    j {trap_handler} // Return imediately
"#,
    trap_handler = sym trap_handler,
);

extern "C" {
    fn _raw_breakpoint_trap_handler();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
