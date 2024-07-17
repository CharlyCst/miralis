//! Miralis ABI
//!
//! While miralis forwards standard SBI calls to the virtualized firmware, miralis do expose its own
//! ABI for the firmware to interact with.
//!
//! The Miralis ABI tries to be compatible with the SBI specification as best as it can.
//! See: https://github.com/riscv-non-isa/riscv-sbi-doc
#![no_std]

use core::arch::asm;
use core::hint;

use miralis_core::abi;

pub mod logger;

pub use log;

// ———————————————————————————— Client Functions ———————————————————————————— //

/// Ask Miralis to exit with a success error code.
pub fn success() -> ! {
    unsafe { miralis_ecall(abi::MIRALIS_SUCCESS_FID).ok() };

    // Loop forever, this should never happen as Miralis will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

/// Ask Miralis to exit with a failure error code.
pub fn failure() -> ! {
    unsafe { miralis_ecall(abi::MIRALIS_FAILURE_FID).ok() };

    // Loop forever, this should never happen as Miralis will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

// ————————————————————————————— Firmware Setup ————————————————————————————— //

/// Configure the firmware entry point and panic handler.
///
/// This macro prepares all the boiler plate required by Miralis's firmware.
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

/// Configure a panic handler for a Miralis firmware.
///
/// The handler uses the Miralis ABI to gracefully exit with an error.
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
unsafe fn miralis_ecall(fid: usize) -> Result<usize, usize> {
    let error: usize;
    let value: usize;

    asm!(
        "ecall",
        in("a6") fid,
        in("a7") abi::MIRALIS_EID,
        out("a0") error,
        out("a1") value,
    );

    if error != 0 {
        Err(error)
    } else {
        Ok(value)
    }
}
