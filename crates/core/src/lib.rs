//! Mirage core definition
//!
//! This crate purpose is to hold core types and constants definitions for user by other crates.
//! In particular, this crate does not hold any code, this is important as not all code is
//! portable, but some of the definitions here can be used in lots of different contexts (such as
//! in Mirage itself, from firmware or external tooling).

#![no_std]

// ———————————————————————————— ABI Definitions ————————————————————————————— //

/// Mirage ABI definitions.
///
/// The Mirage ABI tries to be compatible with the SBI specification as best as it can.
/// See: https://github.com/riscv-non-isa/riscv-sbi-doc//
pub mod abi {
    /// Mirage SBI Extension ID.
    pub const MIRAGE_EID: usize = 0x08475bcd;
    /// Exit with an error.
    pub const MIRAGE_FAILURE_FID: usize = 0;
    /// Exit successfully.
    pub const MIRAGE_SUCCESS_FID: usize = 1;
    /// Logging interface.
    pub const MIRAGE_LOG_FID: usize = 2;

    /// Log level constants, with the same semantic as the `log` crate.
    pub mod log {
        pub const MIRAGE_ERROR: usize = 1;
        pub const MIRAGE_WARN: usize = 2;
        pub const MIRAGE_INFO: usize = 3;
        pub const MIRAGE_DEBUG: usize = 4;
        pub const MIRAGE_TRACE: usize = 5;
    }
}
