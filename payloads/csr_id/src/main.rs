#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    let csrs = ["mvendorid", "marchid", "mimpid", "mhartid"];

    for csr in csrs {
        let (out_csr, expected) = test_csr_id(csr);
        read_test(out_csr, expected);
    }

    success();
}

fn test_csr_id(csr_name: &str) -> (usize, usize) {
    let expected: usize = 0x0;
    let res: usize;

    match csr_name {
        "mvendorid" => unsafe {
            asm!(
                "csrr {0}, mvendorid",
                out(reg) res
            );
        },
        "marchid" => unsafe {
            asm!(
                "csrr {0}, marchid",
                out(reg) res
            );
        },
        "mimpid" => unsafe {
            asm!(
                "csrr {0}, mimpid",
                out(reg) res
            );
        },
        "mhartid" => unsafe {
            asm!(
                "csrr {0}, mhartid",
                out(reg) res
            );
        },
        _ => res = 0x42, // To fail by default if no valid CSR register is found
    };

    (res, expected)
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
