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
    // For now we expose 0 PMPs to the payload, because QEMU supports only 16 PMPs.
    // So we ensure that indeed no PMPs are exposed.
    test_simple_regs();

    test_some_counters_events();

    success();
}

fn test_simple_regs() {
    let secret: usize = 0x42;
    let mut res: usize;

    // Test mcycle
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcycle, {0}",
            "csrr {1}, mcycle",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test minstret
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw minstret, {0}",
            "csrr {1}, minstret",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mcountinhibit
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcountinhibit, {0}",
            "csrr {1}, mcountinhibit",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret & !0b10);

    // Test mcounteren
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mcounteren, {0}",
            "csrr {1}, mcounteren",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);
}

fn test_some_counters_events() {
    let secret: usize = 0x42;
    let mut res: usize;

    // Test mhpmcounter3
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter3, {0}",
            "csrr {1}, mhpmcounter3",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mhpmcounter5
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter5, {0}",
            "csrr {1}, mhpmcounter5",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mhpmcounter7
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmcounter7, {0}",
            "csrr {1}, mhpmcounter7",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mhpmevent3
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent3, {0}",
            "csrr {1}, mhpmevent3",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mhpmevent5
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent5, {0}",
            "csrr {1}, mhpmevent5",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);

    // Test mhpmevent7
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mhpmevent7, {0}",
            "csrr {1}, mhpmevent7",
            in(reg) secret,
            out(reg) res,
        );
    }

    read_test(res, secret);
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
