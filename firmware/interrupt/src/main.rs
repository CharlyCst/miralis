#![no_std]
#![no_main]

use core::arch::asm;

use mirage_abi::{setup_firmware, success};

setup_firmware!(main);

fn main() -> ! {
    log::debug!("Testing mie register");
    test_mie();
    log::debug!("Testing sie register");
    test_sie();
    log::debug!("Testing sie by mie register");
    test_sie_by_mie();
    log::debug!("Done!");
    success();
}

// —————————————————————————————— mie and sie ——————————————————————————————— //

// Test mie with a simple read and write
fn test_mie() {
    let res: usize;
    unsafe {
        asm!(
            "li {0}, 0x42",
            "csrw mie, {0}",
            "csrr {1}, mie",
            out(reg) _,
            out(reg) res,
        );
    }

    assert_eq!(res, 0x42);
}

// Test sie: it should be masked by S-mode bit only
fn test_sie() {
    let sie: usize;
    let mie: usize;
    let value = 0x3ff;
    let masked_value = value & 0x222;

    unsafe {
        asm!(
            "csrw sie, {value}",
            "csrr {sie}, sie",
            "csrr {mie}, mie",
            sie = out(reg) sie,
            mie = out(reg) mie,
            value = in(reg) value,
        );
    }

    assert_eq!(
        sie, masked_value,
        "sie is correctly set to the masked value"
    );
    assert_eq!(mie & 0x222, masked_value, "mie S bits need to be set");
}

// Test sie: writting to mie must be
fn test_sie_by_mie() {
    let res: usize;
    let value = 0x3ff;
    let masked_value = value & 0x222;
    unsafe {
        asm!(
            "csrw mie, {value}",
            "csrr {0}, sie",
            out(reg) res,
            value = in(reg) value,
        );
    }

    assert_eq!(res, masked_value);
}
