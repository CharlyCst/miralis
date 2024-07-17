//! Miralis core definition
//!
//! This crate purpose is to hold core types and constants definitions for user by other crates.
//! In particular, this crate does not hold any code, this is important as not all code is
//! portable, but some of the definitions here can be used in lots of different contexts (such as
//! in Miralis itself, from firmware or external tooling).

#![no_std]

// ———————————————————————————— ABI Definitions ————————————————————————————— //

/// Miralis ABI definitions.
///
/// The Miralis ABI tries to be compatible with the SBI specification as best as it can.
/// See: https://github.com/riscv-non-isa/riscv-sbi-doc//
pub mod abi {
    /// Miralis SBI Extension ID.
    pub const MIRALIS_EID: usize = 0x08475bcd;
    /// Exit with an error.
    pub const MIRALIS_FAILURE_FID: usize = 0;
    /// Exit successfully.
    pub const MIRALIS_SUCCESS_FID: usize = 1;
    /// Logging interface.
    pub const MIRALIS_LOG_FID: usize = 2;

    /// Log level constants, with the same semantic as the `log` crate.
    pub mod log {
        pub const MIRALIS_ERROR: usize = 1;
        pub const MIRALIS_WARN: usize = 2;
        pub const MIRALIS_INFO: usize = 3;
        pub const MIRALIS_DEBUG: usize = 4;
        pub const MIRALIS_TRACE: usize = 5;
    }
}
