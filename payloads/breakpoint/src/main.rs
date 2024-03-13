#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

use mirage_abi::{failure, success};

global_asm!(
    r#"
.text
.align 4
.global _start
_start:
    j {entry}
"#,
    entry = sym entry,
);

extern "C" fn entry() -> ! {
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

// ————————————————————————————— Panic Handler —————————————————————————————— //

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
