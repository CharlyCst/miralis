#![no_std]
#![no_main]

use core::arch::asm;

use miralis_abi::{log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    log::info!("Hello from Misaligned operations firmware");

    let misaligned_address_2_bytes: usize = 0x80400101;
    let misaligned_address_4_bytes: usize = 0x80400201;
    let misaligned_address_8_bytes: usize = 0x80400301;
    let value_2_bytes: u16 = 0xabcd;
    let value_4_bytes: u32 = 0xdeadbeef;
    let value_8_bytes: u64 = 0x1234567887654321;

    // 2 bytes operations
    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!(
        "lh {r}, 0({addr})",
        addr = in(reg) misaligned_address_2_bytes,
        r = out(reg) _,
        )
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!(
        "sh {r}, 0({addr})",
        addr = in(reg) misaligned_address_2_bytes,
        r = in(reg) value_2_bytes,
        )
    }

    // 4 bytes operations
    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!(
        "lw {r}, 0({addr})",
        addr = in(reg) misaligned_address_4_bytes,
        r = out(reg) _,
        )
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!(
        "sw {r}, 0({addr})",
        addr = in(reg) misaligned_address_4_bytes,
        r = in(reg) value_4_bytes,
        )
    }

    // 8 bytes operations
    // Error code 4 - LoadAddrMisaligned
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) misaligned_address_8_bytes,
        r = out(reg) _,
        )
    }

    // Error code 6 - StoreAddrMisaligned
    unsafe {
        asm!(
        "sd {r}, 0({addr})",
        addr = in(reg) misaligned_address_8_bytes,
        r = in(reg) value_8_bytes,
        )
    }

    // Correctness test
    let mut read_value_2: u16;
    unsafe {
        asm!(
        "lh {r}, 0({addr})",
        addr = in(reg) misaligned_address_2_bytes,
        r = out(reg) read_value_2,
        )
    }

    let mut read_value_4: u32;
    unsafe {
        asm!(
        "lw {r}, 0({addr})",
        addr = in(reg) misaligned_address_4_bytes,
        r = out(reg) read_value_4,
        )
    }

    let mut read_value_8: u64;
    unsafe {
        asm!(
        "ld {r}, 0({addr})",
        addr = in(reg) misaligned_address_8_bytes,
        r = out(reg) read_value_8,
        )
    }

    assert_eq!(
        read_value_2, value_2_bytes,
        "Misaligned loads and stores emulation doesn't work properly for 2 bytes"
    );

    assert_eq!(
        read_value_4, value_4_bytes,
        "Misaligned loads and stores emulation doesn't work properly for 4 bytes"
    );

    assert_eq!(
        read_value_8, value_8_bytes,
        "Misaligned loads and stores emulation doesn't work properly for 8 bytes"
    );

    success()
}
