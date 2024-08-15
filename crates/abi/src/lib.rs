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

pub use config_helpers::{is_enabled, parse_usize_or};
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

/// Ask Miralis to end benchmark and print results.
pub fn miralis_end_benchmark() -> ! {
    unsafe { miralis_ecall(abi::MIRALIS_BENCHMARK_FID).ok() };

    // Loop forever, this should never happen as Miralis will terminate the execution before.
    loop {
        hint::spin_loop();
    }
}

// —————————————————————————————— Binary Setup —————————————————————————————— //

/// Configure the binary entry point and panic handler.
///
/// This macro prepares all the boiler plate required by Miralis's firmware or payload.
#[macro_export]
macro_rules! setup_binary {
    ($path:path) => {
        // The assembly entry point
        core::arch::global_asm!(
            r#"
            .text
            .align 4
            .global _start
            _start:
                ld t0, __stack_start
                ld t1, {_stack_size}  // Per-hart stack size

                // compute how much space we need to put before this hart's stack
                add t3, x0, x0       // Initialize offset to zero
                add t4, x0, x0       // Initialize counter to zero
            
            stack_start_loop:
                // First we exit the loop once we made enough iterations (N iterations for hart N)
                bgeu t4, a0, stack_start_done
                add t3, t3, t1       // Add space for one more stack
                addi t4, t4, 1       // Increment counter
                j stack_start_loop

            stack_start_done:
                add t0, t0, t3       // The actual start of our stack
                add t1, t0, t1       // And the end of our stack
                            
                // Zero out the BSS section
                ld t4, __bss_start
                ld t5, __bss_stop
            zero_bss_loop:
                bgeu t4, t5, zero_bss_done
                sd x0, 0(t4)
                addi t4, t4, 8
                j zero_bss_loop
            zero_bss_done:
                // Load the stack pointer and jump into main
                mv sp, t1
                j {entry}

                // Store the address of the stack in memory
                // That way it can be loaded as an absolute value
            .align 8
            __stack_start:
                .dword {_stack_start}
            __bss_start:
                .dword {_bss_start}
            __bss_stop:
                .dword {_bss_stop}
            "#,
            entry = sym _start,
            _stack_start = sym _stack_start,
            _stack_size = sym STACK_SIZE,
            _bss_start = sym _bss_start,
            _bss_stop = sym _bss_stop,
        );

        static STACK_SIZE: usize = $crate::parse_usize_or(
            if $crate::is_enabled!("IS_TARGET_FIRMWARE") {
                option_env!("MIRALIS_TARGET_FIRMWARE_STACK_SIZE")
            } else {
                option_env!("MIRALIS_TARGET_PAYLOAD_STACK_SIZE")
            }
        , 0x8000);

        pub extern "C" fn _start() -> ! {
            // Validate the signature of the entry point.
            let f: fn() -> ! = $path;

            // Initialize logger
            $crate::logger::init();

            f();
        }

        // Defined in the linker script
        extern "C" {
            pub(crate) static _stack_start: u8;
            pub(crate) static _bss_start: u8;
            pub(crate) static _bss_stop: u8;
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

// ——————————————————————————————— Constants ———————————————————————————————— //

/// Number of iterations to be used by benchmark firmware.
pub const BENCHMARK_NB_ITER: usize = parse_usize_or(option_env!("MIRALIS_BENCHMARK_NB_ITER"), 1);
