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
    /// Benchmark prints and exit.
    pub const MIRALIS_BENCHMARK_FID: usize = 3;
    /// Returns the performance counters managed by Miralis.
    pub const MIRALIS_READ_COUNTERS_FID: usize = 4;

    /// Log level constants, with the same semantic as the `log` crate.
    pub mod log {
        pub const MIRALIS_ERROR: usize = 1;
        pub const MIRALIS_WARN: usize = 2;
        pub const MIRALIS_INFO: usize = 3;
        pub const MIRALIS_DEBUG: usize = 4;
        pub const MIRALIS_TRACE: usize = 5;
    }
}

pub mod abi_protect_payload {
    use crate::abi::MIRALIS_EID;

    /// Protect payload SBI Extension ID.
    pub const MIRALIS_PROTECT_PAYLOAD_EID: usize = MIRALIS_EID + 1;
    /// Ecall to lock the payload
    pub const MIRALIS_PROTECT_PAYLOAD_LOCK_FID: usize = 0x1;
}

// ———————————————————————————— RISCV SBI Definitions ————————————————————————————— //

// Constants to idenfity the various SBI codes we use in Miralis
// Documentation available here: https://github.com/riscv-non-isa/riscv-sbi-doc

pub mod sbi_codes {

    // SBI return codes used in Miralis
    pub const SBI_ERR_DENIED: usize = (-4_i64) as usize;

    // SBI EIDs and FIDs
    /// The debug console extension defines a generic mechanism for boot-time early prints.
    pub const SBI_DEBUG_CONSOLE_EXTENSION_EID: usize = 0x4442434E;
}
