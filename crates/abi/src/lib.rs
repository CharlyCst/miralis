//! Miralis ABI
//!
//! While miralis forwards standard SBI calls to the virtualized firmware, miralis do expose its own
//! ABI for the firmware to interact with.
//!
//! The Miralis ABI tries to be compatible with the SBI specification as best as it can.
//! See: https://github.com/riscv-non-isa/riscv-sbi-doc
#![no_std]

use core::fmt::{self, Write};
use core::hint;

pub use config_helpers::{is_enabled, parse_usize_or};
use log::Level;
use miralis_core::abi;

use crate::logger::StackBuffer;

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

/// Ask Miralis to log a string with the provided log level.
pub fn miralis_log(level: Level, message: &str) {
    // Prepare ecall arguments
    let fid = abi::MIRALIS_LOG_FID;
    let level = match level {
        log::Level::Error => abi::log::MIRALIS_ERROR,
        log::Level::Warn => abi::log::MIRALIS_WARN,
        log::Level::Info => abi::log::MIRALIS_INFO,
        log::Level::Debug => abi::log::MIRALIS_DEBUG,
        log::Level::Trace => abi::log::MIRALIS_TRACE,
    };
    let addr = message.as_ptr() as usize;
    let len = message.len();

    unsafe { ecall3(abi::MIRALIS_EID, fid, level, addr, len).expect("Failed to log") };
}

/// Ask Miralis to log a formatted string with the provided log level.
pub fn miralis_log_fmt(level: Level, args: fmt::Arguments) {
    let mut buff: StackBuffer<300> = StackBuffer::new();
    buff.write_fmt(args).unwrap();
    miralis_log(level, buff.as_str());
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

/// # Safety
/// This function will always panic if not executed on a riscv64 architecture
#[inline]
#[cfg(not(target_arch = "riscv64"))]
pub unsafe fn ecall3(
    _eid: usize,
    _fid: usize,
    _a0: usize,
    _a1: usize,
    _a2: usize,
) -> Result<usize, usize> {
    panic!("Tried to use `policy ecall` on non RISC-V archiecture");
}

/// Execute an ecall with 3 arguments.
/// SAFETY: Miralis might panic if the fid or eid are not recognized.
#[inline]
#[cfg(target_arch = "riscv64")]
pub unsafe fn ecall3(
    eid: usize,
    fid: usize,
    a0: usize,
    a1: usize,
    a2: usize,
) -> Result<usize, usize> {
    let error: usize;
    let value: usize;

    core::arch::asm!(
    "ecall",
    inout("a0") a0 => error,
    inout("a1") a1 => value,
    in("a2") a2,
    in("a6") fid,
    in("a7") eid,
    );

    if error != 0 {
        Err(error)
    } else {
        Ok(value)
    }
}

#[inline]
unsafe fn miralis_ecall(fid: usize) -> Result<usize, usize> {
    ecall3(abi::MIRALIS_EID, fid, 0, 0, 0)
}

// ——————————————————————————————— Constants ———————————————————————————————— //

/// Number of iterations to be used by benchmark firmware.
pub const BENCHMARK_NB_ITER: usize = parse_usize_or(option_env!("MIRALIS_BENCHMARK_NB_ITER"), 1);
