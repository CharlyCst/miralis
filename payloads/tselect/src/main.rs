#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
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
