//! Test protect payload policy
//!
//! This payload serve as test payload for the protect payload policy. It must be used with the firmware test_protect_payload_firmware only.
//! These two components together make sure we enforce the protect payload policy correctly.
#![no_std]
#![no_main]
#![feature(start)]

// ———————————————————————————————— Register structure ———————————————————————————————— //

#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
struct Regs {
    registers: [usize; 32],
}

impl Regs {
    pub fn new() -> Self {
        Regs { registers: [0; 32] }
    }
}

macro_rules! save_registers {
    ($regs:expr) => {
            asm!(
                "sd x0, 0*8({0})",
                "sd x1, 1*8({0})",
                "sd x2, 2*8({0})",
                "sd x3, 3*8({0})",
                "sd x4, 4*8({0})",
                "sd x5, 5*8({0})",
                "sd x6, 6*8({0})",
                "sd x7, 7*8({0})",
                "sd x8, 8*8({0})",
                "sd x9, 9*8({0})",
                "sd x10, 10*8({0})",
                "sd x11, 11*8({0})",
                "sd x12, 12*8({0})",
                "sd x13, 13*8({0})",
                "sd x14, 14*8({0})",
                "sd x15, 15*8({0})",
                "sd x16, 16*8({0})",
                "sd x17, 17*8({0})",
                "sd x18, 18*8({0})",
                "sd x19, 19*8({0})",
                "sd x20, 20*8({0})",
                "sd x21, 21*8({0})",
                "sd x22, 22*8({0})",
                "sd x23, 23*8({0})",
                "sd x24, 24*8({0})",
                "sd x25, 25*8({0})",
                "sd x26, 26*8({0})",
                "sd x27, 27*8({0})",
                "sd x28, 28*8({0})",
                "sd x29, 29*8({0})",
                "sd x30, 30*8({0})",
                "sd x31, 31*8({0})",
                in(reg) &mut $regs.registers,
            );

    };
}
// ———————————————————————————————— Guest OS ———————————————————————————————— //

use core::arch::asm;

use miralis_abi::{lock_payload, log, setup_binary, success};

setup_binary!(main);

fn main() -> ! {
    // Say hello
    log::info!("Hello from test protect payload payload");

    // Lock payload to firmware
    lock_payload();

    const NB_TRIES: usize = 75;

    for _ in 0..NB_TRIES {
        // Make sure the firmware can't overwrite the set of registers
        assert!(
            test_same_register_after_illegal_instruction(),
            "Test same register after trap failed"
        );

        // Make sure the firmware can't overwrite a memory region
        assert!(
            test_same_region_after_trap(),
            "Test same region after trap failed"
        );

        // Make sure the ecall parameters goes through
        assert!(test_ecall_rule(), "Ecall test failed");
    }

    // and exit
    success();
}

fn test_same_register_after_illegal_instruction() -> bool {
    let mut regs_before = Regs::new();
    let mut regs_after = Regs::new();

    unsafe {
        save_registers!(regs_before);
        asm!("csrw mscratch, zero");
        save_registers!(regs_after);
    }

    regs_before == regs_after
}

fn test_same_region_after_trap() -> bool {
    let address: *const usize = 0x80400000 as *const usize;
    let value: usize;

    unsafe {
        value = *address;
    }

    value != 60
}

fn test_ecall_rule() -> bool {
    let ret_value_1: usize;
    let ret_value_2: usize;

    unsafe {
        asm!(
        "li a0, 60",
        "li a1, 60",
        "li a2, 60",
        "li a3, 60",
        "li a4, 60",
        "li a5, 60",
        "li x17, 0x08475bd0",
        "ecall",
        out("x10") ret_value_1,
        out("x11") ret_value_2,
        out("x12") _,
        out("x13") _,
        out("x14") _,
        out("x15") _,
        out("x16") _,
        out("x17") _,
        );
    }

    ret_value_1 == 61 && ret_value_2 == 62
}
