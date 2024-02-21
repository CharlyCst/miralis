#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::usize;

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
    let expected: usize = 0x0;
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrr {1}, mimpid",
            in(reg) expected,
            out(reg) res,
        );
    }

    if res == expected {
        success();
    }

    panic!();
}

#[inline(always)]
fn success() {
    unsafe {
        asm!("ecall");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}
