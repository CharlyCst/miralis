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

    read_test(res,secret_cfg);

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

    read_test(res,secret_cfg);

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

    read_test(res,0);
    
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

    read_test(res,secret_addr);

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

    read_test(res,secret_addr);

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

    read_test(res,0);

    success();
}

fn read_test(out_csr: usize, expected: usize) {
    assert_eq!(out_csr, expected);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    failure();
}
