//! Configuration constants
//!
//! The constants in this files are parsed from the Mirage configuration file (passed through
//! environment variables by the runner during Mirage build).

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Helper macro to check is boolean choice is enabled by the configuration, defaulting to yes.
///
/// The current implementation works around the limitation of const functions in rust at the
/// time of writing.
macro_rules! is_enabled {
    ($env_var: tt) => {
        match option_env!($env_var) {
            Some(env_var) => match env_var.as_bytes() {
                b"false" => false,
                _ => true,
            },
            None => true,
        }
    };
}

// ———————————————————————— Configuration Parameters ———————————————————————— //

/// Weather the platform supports S mode.
pub const HAS_S_MODE: bool = is_enabled!("MIRAGE_PLATFORM_S_MODE");

/// The desired log level.
pub const LOG_LEVEL: Option<&'static str> = option_env!("MIRAGE_LOG_LEVEL");

/// The maximum number of firmware exits before quitting.
pub const MAX_FIRMWARE_EXIT: Option<&'static str> = option_env!("MIRAGE_DEBUG_MAX_FIRMWARE_EXITS");
