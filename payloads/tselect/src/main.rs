#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::usize;

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
    let secret1: usize = 0x42;
    let mut res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw tselect, {0}",
            "csrr {1}, tselect",
            in(reg) secret1,
            out(reg) res,
        );
    }

    read_test(res, 0);

    success();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
