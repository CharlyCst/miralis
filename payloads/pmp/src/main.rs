#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_payload, success};

setup_payload!(main);

fn main() -> ! {
    // For now we expose 0 PMPs to the payload, because QEMU supports only 16 PMPs.
    // So we ensure that indeed no PMPs are exposed.
    test_0_pmp();

    success();
}

fn test_0_pmp() {
    let addr: usize = 0x00000fffffffff42;
    let cfg: usize = 0b00000111;
    let mut res: usize;

    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) cfg,
            out(reg) res,
        );
    }

    // PMPs are WARL, so when no PMPs are implemented we must read 0
    assert_eq!(res, 0);

    unsafe {
        asm!(
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) addr,
            out(reg) res,
        );
    }

    // PMPs are WARL, so when no PMPs are implemented we must read 0
    assert_eq!(res, 0);
}

// Not yet tested because QEMU exposes only 16 PMPs, se Mirage exposes 0 to the payload.
#[allow(unused)]
fn test_16_pmp() {
    let secret_addr: usize = 0x0000000000000042;
    let secret_cfg: usize = 0b00000111;
    let mut res: usize;

    // Test normal write to config
    unsafe {
        asm!(
            "li {0}, 0b00000111",
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, secret_cfg);

    // Test invalid write to config
    unsafe {
        asm!(
            "li {0}, 0b01100111",
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, secret_cfg);

    // Test out of range write to config : for 16 pmp
    unsafe {
        asm!(
            "li {0}, 0b01100111",
            "csrw pmpcfg4, {0}",
            "csrr {1}, pmpcfg4",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, 0);

    // Test normal write to address
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, secret_addr);

    // Test invalid write to address
    unsafe {
        asm!(
            "li {0}, 0xf000000000000042",
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, secret_addr);

    // Test out of range write to address : for 16 pmp
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw pmpaddr17, {0}",
            "csrr {1}, pmpaddr17",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }

    read_test(res, 0);
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}
