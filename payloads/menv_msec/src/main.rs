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
    let secret: usize = 0x42;
    let mut res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw menvcfg, {0}",
            "csrr {1}, menvcfg",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mseccfg, {0}",
            "csrr {1}, mseccfg",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    success()
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
