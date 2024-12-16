//! Configuration constants
//!
//! The constants in this file are parsed from the Miralis configuration file (passed through
//! environment variables by the runner during Miralis build).

use config_helpers::{
    is_enabled, is_enabled_default_false, parse_str_list, parse_str_or, parse_usize,
    parse_usize_or, str_list_len,
};

use crate::platform::{Plat, Platform};

// ———————————————————————— Configuration Parameters ———————————————————————— //

/// Maximum number of PMP exposed by the vCPU, no limit if None.
pub const VCPU_MAX_PMP: Option<usize> = parse_usize(option_env!("MIRALIS_VCPU_MAX_PMP"));

/// The desired log level.
pub const LOG_LEVEL: Option<&'static str> = option_env!("MIRALIS_LOG_LEVEL");

/// If colors in logs are enabled.
pub const LOG_COLOR: bool = is_enabled!("MIRALIS_LOG_COLOR");

/// The maximum number of firmware exits before quitting.
pub const MAX_FIRMWARE_EXIT: Option<usize> =
    parse_usize(option_env!("MIRALIS_DEBUG_MAX_FIRMWARE_EXITS"));

/// Log error
pub const LOG_ERROR: &[&str; str_list_len(option_env!("MIRALIS_LOG_ERROR"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_ERROR"));

/// Log warn
pub const LOG_WARN: &[&str; str_list_len(option_env!("MIRALIS_LOG_WARN"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_WARN"));

/// Log info
pub const LOG_INFO: &[&str; str_list_len(option_env!("MIRALIS_LOG_INFO"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_INFO"));

/// Log debug
pub const LOG_DEBUG: &[&str; str_list_len(option_env!("MIRALIS_LOG_DEBUG"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_DEBUG"));

/// Log trace
pub const LOG_TRACE: &[&str; str_list_len(option_env!("MIRALIS_LOG_TRACE"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_TRACE"));

/// The target platform
pub const PLATFORM_NAME: &str = parse_str_or(option_env!("MIRALIS_PLATFORM_NAME"), "qemu_virt");

/// The expected number of harts.
pub const PLATFORM_NB_HARTS: usize = {
    const MAX_NB_HARTS: usize = parse_usize_or(option_env!("MIRALIS_PLATFORM_NB_HARTS"), 1);
    if MAX_NB_HARTS < Plat::NB_HARTS {
        MAX_NB_HARTS
    } else {
        Plat::NB_HARTS
    }
};

/// Delegate performance counters
pub const DELEGATE_PERF_COUNTER: bool = is_enabled_default_false!("MIRALIS_DELEGATE_PERF_COUNTER");

/// Boot hart id
#[allow(dead_code)] // Because rust analyzer doesn't understand that it is used in metals.rs
pub const PLATFORM_BOOT_HART_ID: usize =
    parse_usize_or(option_env!("MIRALIS_PLATFORM_BOOT_HART_ID"), 0);

/// Whether any benchmark is enabled
pub const BENCHMARK: bool = is_enabled_default_false!("MIRALIS_BENCHMARK");

/// Whether print in csv format or not
pub const BENCHMARK_CSV_FORMAT: bool = is_enabled!("MIRALIS_BENCHMARK_CSV_FORMAT");

/// Whether execution time benchmarking is enabled
pub const BENCHMARK_TIME: bool = is_enabled!("MIRALIS_BENCHMARK_TIME");

/// Whether instruction count benchmarking is enabled
pub const BENCHMARK_INSTRUCTION: bool = is_enabled!("MIRALIS_BENCHMARK_INSTRUCTION");

/// Whether count or not total number of exits
pub const BENCHMARK_NB_EXITS: bool = is_enabled!("MIRALIS_BENCHMARK_NB_EXISTS");

/// Whether count or not number of firmware exits
pub const BENCHMARK_NB_FIRMWARE_EXITS: bool = is_enabled!("MIRALIS_BENCHMARK_NB_FIRMWARE_EXITS");

/// Whether count or not number of world switches
pub const BENCHMARK_WORLD_SWITCHES: bool = is_enabled!("MIRALIS_BENCHMARK_WORLD_SWITCHES");

/// Start address of Miralis
pub const TARGET_START_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_START_ADDRESS"), 0x80000000);

/// Start address of firmware
pub const TARGET_FIRMWARE_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_FIRMWARE_ADDRESS"), 0x80200000);

/// Start address of the payload
pub const TARGET_PAYLOAD_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_PAYLOAD_ADDRESS"), 0x80400000);

/// The stack size for each Miralis thread (one per hart)
pub const TARGET_STACK_SIZE: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_STACK_SIZE"), 0x8000);

/// The choosen policy name
///
/// For now this variable is unused, but we keep it still to force re-compilation when the policy
/// name changes. We can get rid of it once it becomes possible to track dependencies on
/// environment variables from procedural macros.
///
/// See https://github.com/rust-lang/rust/issues/99515
#[allow(unused)]
pub const POLICY_NAME: &str = parse_str_or(option_env!("MIRALIS_POLICY_NAME"), "default_policy");

/// Size of the payload to hash
pub const PAYLOAD_HASH_SIZE: usize = parse_usize_or(option_env!("PAYLOAD_HASH_SIZE"), 0x2000000);
