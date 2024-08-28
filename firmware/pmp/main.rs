#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    let secret_addr: usize = 0x0000000000000042;
    let secret_cfg: usize = 0b00000111;
    let mut res: usize;

    // Test normal write to config
    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) secret_cfg,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_cfg, "Could not write pmpcfg0");

    // Test invalid write to config
    unsafe {
        asm!(
            "csrw pmpcfg0, {0}",
            "csrr {1}, pmpcfg0",
            in(reg) 0b01100111,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_cfg, "Could set invalid bits in pmpcfg0");

    // Test out of range write to config (with 8 PMP)
    unsafe {
        asm!(
            "csrw pmpcfg4, {0}",
            "csrr {1}, pmpcfg4",
            in(reg) 0b01100111,
            out(reg) res,
        );
    }
    assert_eq!(res, 0, "Could write to unimplemented PMP");

    // Test normal write to address
    unsafe {
        asm!(
            "csrw pmpaddr0, {0}",
            "csrr {1}, pmpaddr0",
            in(reg) 0x42,
            out(reg) res,
        );
    }
    assert_eq!(res, secret_addr, "Could not write to pmpaddr0");

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
    assert_eq!(res, secret_addr, "Could write an invalid address");

    // Test out of range write to address : for 16 pmp
    unsafe {
        asm!(
            "csrw pmpaddr17, {0}",
            "csrr {1}, pmpaddr9",
            in(reg) 0x42,
            out(reg) res,
        );
    }
    assert_eq!(res, 0, "Could write to an unimplemented PMP");

    success();
}
