//! Miralis Configuration
//!
//! This crate hosts the environment variables used to configure Miralis, the choosen configuration
//! values, as well as helpers to
//! parse the variables values at compile time.

#![no_std]

pub mod helper;
use helper::*;

// ———————————————————————————————— Logging ————————————————————————————————— //

/// The desired log level.
pub const LOG_LEVEL: Option<&'static str> = option_env!("MIRALIS_LOG_LEVEL");
pub const LOG_LEVEL_ENV: &str = "MIRALIS_LOG_LEVEL";

/// If colors in logs are enabled.
pub const LOG_COLOR: bool = is_enabled!("MIRALIS_LOG_COLOR");
pub const LOG_COLOR_ENV: &str = "MIRALIS_LOG_COLOR";

/// Log error
pub const LOG_ERROR: &[&str; str_list_len(option_env!("MIRALIS_LOG_ERROR"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_ERROR"));
pub const LOG_ERROR_ENV: &str = "MIRALIS_LOG_ERROR";

/// Log warn
pub const LOG_WARN: &[&str; str_list_len(option_env!("MIRALIS_LOG_WARN"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_WARN"));
pub const LOG_WARN_ENV: &str = "MIRALIS_LOG_WARN";

/// Log info
pub const LOG_INFO: &[&str; str_list_len(option_env!("MIRALIS_LOG_INFO"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_INFO"));
pub const LOG_INFO_ENV: &str = "MIRALIS_LOG_INFO";

/// Log debug
pub const LOG_DEBUG: &[&str; str_list_len(option_env!("MIRALIS_LOG_DEBUG"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_DEBUG"));
pub const LOG_DEBUG_ENV: &str = "MIRALIS_LOG_DEBUG";

/// Log trace
pub const LOG_TRACE: &[&str; str_list_len(option_env!("MIRALIS_LOG_TRACE"))] =
    &parse_str_list(option_env!("MIRALIS_LOG_TRACE"));
pub const LOG_TRACE_ENV: &str = "MIRALIS_LOG_TRACE";

// ————————————————————————————————— Debug —————————————————————————————————— //

/// The maximum number of firmware exits before quitting.
pub const MAX_FIRMWARE_EXIT: Option<usize> =
    parse_usize(option_env!("MIRALIS_DEBUG_MAX_FIRMWARE_EXITS"));
pub const MAX_FIRMWARE_EXIT_ENV: &str = "MIRALIS_DEBUG_MAX_FIRMWARE_EXITS";

/// Number of iteration for our benchmarks
pub const BENCHMARK_NB_ITER: Option<usize> = parse_usize(option_env!("MIRALIS_BENCHMARK_NB_ITER"));
pub const BENCHMARK_NB_ITER_ENV: &str = "MIRALIS_BENCHMARK_NB_ITER";

// —————————————————————————————————— vCPU —————————————————————————————————— //

/// Maximum number of PMP exposed by the vCPU, no limit if None.
pub const VCPU_MAX_PMP: Option<usize> = parse_usize(option_env!("MIRALIS_VCPU_MAX_PMP"));
pub const VCPU_MAX_PMP_ENV: &str = "MIRALIS_VCPU_MAX_PMP";

// ———————————————————————————————— Platform ———————————————————————————————— //

/// The target platform
pub const PLATFORM_NAME: &str = parse_str_or(option_env!("MIRALIS_PLATFORM_NAME"), "qemu_virt");
pub const PLATFORM_NAME_ENV: &str = "MIRALIS_PLATFORM_NAME";

/// The expected number of harts.
pub const PLATFORM_NB_HARTS: usize = parse_usize_or(option_env!("MIRALIS_PLATFORM_NB_HARTS"), 1);
pub const PLATFORM_NB_HARTS_ENV: &str = "MIRALIS_PLATFORM_NB_HARTS";

/// Delegate performance counters
pub const DELEGATE_PERF_COUNTER: bool = is_enabled_default_false!("MIRALIS_DELEGATE_PERF_COUNTER");
pub const DELEGATE_PERF_COUNTER_ENV: &str = "MIRALIS_DELEGATE_PERF_COUNTER";

/// Boot hart id
pub const PLATFORM_BOOT_HART_ID: usize =
    parse_usize_or(option_env!("MIRALIS_PLATFORM_BOOT_HART_ID"), 0);
pub const PLATFORM_BOOT_HART_ID_ENV: &str = "MIRALIS_PLATFORM_BOOT_HART_ID";

// ————————————————————————————————— Target ————————————————————————————————— //

/// Start address of Miralis
pub const TARGET_START_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_START_ADDRESS"), 0x80000000);
pub const TARGET_START_ADDRESS_ENV: &str = "MIRALIS_TARGET_START_ADDRESS";

/// Start address of firmware
pub const TARGET_FIRMWARE_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_FIRMWARE_ADDRESS"), 0x80200000);
pub const TARGET_FIRMWARE_ADDRESS_ENV: &str = "MIRALIS_TARGET_FIRMWARE_ADDRESS";

/// Start address of the payload
pub const TARGET_PAYLOAD_ADDRESS: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_PAYLOAD_ADDRESS"), 0x80400000);
pub const TARGET_PAYLOAD_ADDRESS_ENV: &str = "MIRALIS_TARGET_PAYLOAD_ADDRESS";

/// The stack size for each Miralis thread (one per hart)
pub const TARGET_STACK_SIZE: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_STACK_SIZE"), 0x8000);
pub const TARGET_STACK_SIZE_ENV: &str = "MIRALIS_TARGET_STACK_SIZE";

/// The stack size for each firmware thread (one per hart)
pub const TARGET_FIRMWARE_STACK_SIZE: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_FIRMWARE_STACK_SIZE"), 0x8000);
pub const TARGET_FIRMWARE_STACK_SIZE_ENV: &str = "MIRALIS_TARGET_FIRMWARE_STACK_SIZE";

/// The stack size for each payload thread (one per hart)
pub const TARGET_PAYLOAD_STACK_SIZE: usize =
    parse_usize_or(option_env!("MIRALIS_TARGET_PAYLOAD_STACK_SIZE"), 0x8000);
pub const TARGET_PAYLOAD_STACK_SIZE_ENV: &str = "MIRALIS_TARGET_PAYLOAD_STACK_SIZE";

// ———————————————————————————————— Modules ————————————————————————————————— //

/// The list of enabled modules.
pub const MODULES: &[&str; str_list_len(option_env!("MIRALIS_MODULES"))] =
    &parse_str_list(option_env!("MIRALIS_MODULES"));
pub const MODULES_ENV: &str = "MIRALIS_MODULES";
