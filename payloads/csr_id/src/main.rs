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
    let csrs = ["mvendorid", "marchid", "mimpid", "mhartid"];

    for csr in csrs {
        let (out_csr, expected) = test_csr_id(csr);
        read_test(out_csr, expected);
    }

    success();
    panic!();
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
        _ => res = 0x42, //TO FAIL BY DEFAULT IF NO VALID CSR REGISTER IS FOUND
    };

    (res, expected)
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
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
