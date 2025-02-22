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

// ———————————————————————————— RISCV SBI Definitions ————————————————————————————— //

// Constants to idenfity the various SBI codes we use in Miralis
// Documentation available here: https://github.com/riscv-non-isa/riscv-sbi-doc

pub mod sbi_codes {

    // SBI return codes used in Miralis
    pub const SBI_ERR_DENIED: usize = (-4_i64) as usize;

    pub const SBI_SUCCESS: usize = 0x0;

    // SBI EIDs and FIDs
    /// The debug console extension defines a generic mechanism for boot-time early prints.
    pub const SBI_DEBUG_CONSOLE_EXTENSION_EID: usize = 0x4442434E;

    /// The SBI_TIMER_EID replaces legacy timer extension (EID #0x00). It follows the new calling convention defined in v0.2.
    pub const SBI_TIMER_EID: usize = 0x54494d45;

    /// Programs the clock for next event after stime_value time. stime_value is in absolute time. This function must clear the pending timer interrupt bit as well.
    /// If the supervisor wishes to clear the timer interrupt without scheduling the next timer event, it can either request a timer interrupt infinitely far into the future (i.e., (uint64_t)-1), or it can instead mask the timer interrupt by clearing sie.STIE CSR bit.
    pub const SBI_TIMER_FID: usize = 0x0;

    /// This extension replaces the legacy extension (EID #0x04). The other IPI related legacy extension(0x3)
    /// is deprecated now. All the functions in this extension follow the hart_mask as defined in the binary
    /// encoding section.
    pub const IPI_EXTENSION_EID: usize = 0x735049;
    /// Send an inter-processor interrupt to all the harts defined in hart_mask. Interprocessor interrupts
    // manifest at the receiving harts as the supervisor software interrupts.
    pub const SEND_IPI_FID: usize = 0x0;

    /// This extension defines all remote fence related functions and replaces the legacy extensions (EIDs
    /// #0x05 - #0x07). All the functions follow the hart_mask as defined in binary encoding section. Any
    /// function wishes to use range of addresses (i.e. start_addr and size), have to abide by the below
    /// constraints on range parameters.
    pub const RFENCE_EXTENSION_EID: usize = 0x52464E43;
    /// Instructs remote harts to execute FENCE.I instruction.
    pub const REMOTE_FENCE_I_FID: usize = 0x0;
    /// Instructs the remote harts to execute one or more SFENCE.VMA instructions, covering the range of
    /// virtual addresses between start and size.
    pub const REMOTE_FENCE_VMA_FID: usize = 0x1;

    pub fn is_timer_request(fid: usize, eid: usize) -> bool {
        fid == SBI_TIMER_FID && eid == SBI_TIMER_EID
    }

    pub fn is_ipi_request(fid: usize, eid: usize) -> bool {
        fid == SEND_IPI_FID && eid == IPI_EXTENSION_EID
    }

    pub fn is_i_fence_request(fid: usize, eid: usize) -> bool {
        fid == REMOTE_FENCE_I_FID && eid == RFENCE_EXTENSION_EID
    }

    pub fn is_vma_request(fid: usize, eid: usize) -> bool {
        fid == REMOTE_FENCE_VMA_FID && eid == RFENCE_EXTENSION_EID
    }
}
