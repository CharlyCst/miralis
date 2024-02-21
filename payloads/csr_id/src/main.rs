#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::usize;
use core::fmt;

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
    let csrs = [
        "mvendorid",
        "marchid",
        "mimpd"
    ];

    for csr in csrs{
        let(out_csr,expected) = test_csr_id(csr);
        read_test(out_csr, expected);
    }


    success();
    panic!();
}

fn test_csr_id(csr_name : &str) -> (usize,usize){
    let expected: usize = 0x0;
    let res: usize;
    let strdaad = "";
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrr {1}, csr_name",
            in(reg) expected,
            out(reg) res
        );
    }

    (res, expected)
}

fn read_test(out_csr: usize, expected: usize){
    assert_eq!(out_csr, expected);
} 

fn check(in_rs1: usize, in_csr: usize, out_csr: usize, out_rd: usize) {
    assert_eq!(out_csr, in_rs1);
    assert_eq!(out_rd, in_csr);
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
