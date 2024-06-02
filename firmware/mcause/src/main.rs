#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    let secret1: usize = 0x42;
    let mut res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcause, {0}",
            "csrr {1}, mcause",
            in(reg) secret1,
            out(reg) res,
        );
    }

    read_test(res, 0);

    let secret2: usize = 0x9;

    unsafe {
        asm!(
            "li {0}, 0x9",
            "csrw mcause, {0}",
            "csrr {1}, mcause",
            in(reg) secret2,
            out(reg) res,
        );
    }

    read_test(res, secret2);

    unsafe {
        asm!(
            "li {0}, 0x8000000000000009",
            "csrw mcause, {0}",
            "csrr {1}, mcause",
            in(reg) secret2,
            out(reg) res,
        );
    }

    read_test(res, secret2);

    success();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
