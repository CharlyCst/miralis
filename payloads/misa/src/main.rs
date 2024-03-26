#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    let secret1: usize = 0x8000000000141129;
    let mut res: usize;
    unsafe {
        asm!(
            "li {0}, 0x8000000000141129",
            "csrw misa, {0}",
            "csrr {1}, misa",
            in(reg) secret1,
            out(reg) res,
        );
    }

    read_test(res, secret1);

    unsafe {
        asm!(
            "li {0}, 0x800000000FFFFFFF",
            "csrw misa, {0}",
            "csrr {1}, misa",
            in(reg) secret1,
            out(reg) res,
        );
    }

    read_test(res, secret1);

    unsafe {
        asm!(
            "li {0}, 0x0000000000141129",
            "csrw misa, {0}",
            "csrr {1}, misa",
            in(reg) secret1,
            out(reg) res,
        );
    }

    read_test(res, secret1);

    success();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
