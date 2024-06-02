//! Mirage ABI
//!
//! While mirage forwards standard SBI calls to the virtualized firmware, mirage do expose its own
//! ABI for the firmware to interact with.
//!
//! The Mirage ABI tries to be compatible with the SBI specification as best as it can.
//! See: https://github.com/riscv-non-isa/riscv-sbi-doc
#![no_std]

use core::arch::asm;
use core::hint;

use mirage_core::abi;

pub mod logger;

pub use log;

// ———————————————————————————— Client Functions ———————————————————————————— //

/// Ask Mirage to exit with a success error code.
pub fn success() -> ! {
    unsafe { mirage_ecall(abi::MIRAGE_SUCCESS_FID).ok() };

    // Loop forever, this should never happen as Mirage will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

/// Ask Mirage to exit with a failure error code.
pub fn failure() -> ! {
    unsafe { mirage_ecall(abi::MIRAGE_FAILURE_FID).ok() };

    // Loop forever, this should never happen as Mirage will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

// ————————————————————————————— Firmware Setup ————————————————————————————— //

/// Configure the firmware entry point and panic handler.
///
/// This macro prepares all the boiler plate required by Mirage's firmware.
#[macro_export]
macro_rules! setup_firmware {
    ($path:path) => {
        // The assembly entry point
        core::arch::global_asm!(
            r#"
            .text
            .align 4
            .global _start
            _start:
                // Load the stack pointer and jump into main
                ld sp, __stack_top
                j {entry}

                // Store the address of the stack in memory
                // That way it can be loaded as an absolute value
                __stack_top:
                    .dword {stack_top}
            "#,
            entry = sym _firmware_start,
            stack_top = sym _stack_top,
        );

        pub extern "C" fn _firmware_start() -> ! {
            // Validate the signature of the entry point.
            let f: fn() -> ! = $path;

            // Initialize logger
            $crate::logger::init();

            f();
        }

        // Defined in the linker script
        extern "C" {
            pub(crate) static _stack_top: u8;
        }

        // Also include the panic handler
        $crate::firmware_panic!();
    };
}

/// Configure a panic handler for a Mirage firmware.
///
/// The handler uses the Mirage ABI to gracefully exit with an error.
#[macro_export]
macro_rules! firmware_panic {
    () => {
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::log::error!("Firmware: {:#?} ", info);
            $crate::failure();
        }
    };
}

// ————————————————————————————————— Utils —————————————————————————————————— //

#[inline]
unsafe fn mirage_ecall(fid: usize) -> Result<usize, usize> {
    let error: usize;
    let value: usize;

    asm!(
        "ecall",
        in("a6") fid,
        in("a7") abi::MIRAGE_EID,
        out("a0") error,
        out("a1") value,
    );

    if error != 0 {
        Err(error)
    } else {
        Ok(value)
    }
}
