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

// ————————————————————————————— String Parsing ————————————————————————————— //
// Required to parse environment variables at compile time.
// Can be removed once usize::from_str_radix stabilized as const, hopefully soon.
// See https://github.com/rust-lang/rust/pull/124941
//
// Source (and license), adapted for Mirage:
// https://gist.github.com/DutchGhost/d8604a3c796479777fe9f5e25d855cfd
// —————————————————————————————————————————————————————————————————————————— //

const fn parse_byte(b: u8, pow10: usize) -> usize {
    let r = b - 48; // Remove ascii offset

    if r > 9 {
        panic!("Failed to parse config: expected usize")
    } else {
        (r as usize) * pow10
    }
}

const POW10: [usize; 20] = {
    let mut array = [0; 20];
    let mut current = 1;

    let mut index = 20;

    loop {
        index -= 1;
        array[index] = current;

        if index == 0 {
            break;
        }

        current *= 10;
    }

    array
};

const fn parse(env_var: Option<&str>) -> Option<usize> {
    let Some(env_var) = env_var else {
        return None;
    };

    let bytes = env_var.as_bytes();
    let mut result: usize = 0;

    let len = bytes.len();

    // Start at the correct index of the table,
    // (skip the power's that are too large)
    let mut index_const_table = POW10.len().wrapping_sub(len);
    let mut index = 0;

    while index < env_var.len() {
        let pow = POW10[index_const_table];
        result += parse_byte(bytes[index], pow);

        index += 1;
        index_const_table += 1;
    }

    Some(result)
}

// ———————————————————————— Configuration Parameters ———————————————————————— //

/// Weather the vCPU exposes S-mode.
pub const VCPU_S_MODE: bool = is_enabled!("MIRAGE_VCPU_S_MODE");

/// Maximum number of PMP exposed by the vCPU, no limit if None.
pub const VCPU_MAX_PMP: Option<usize> = parse(option_env!("MIRAGE_VCPU_MAX_PMP"));

/// The desired log level.
pub const LOG_LEVEL: Option<&'static str> = option_env!("MIRAGE_LOG_LEVEL");

/// If colors in logs are enabled
pub const LOG_COLOR: bool = is_enabled!("MIRAGE_LOG_COLOR");

/// The maximum number of firmware exits before quitting.
pub const MAX_FIRMWARE_EXIT: Option<usize> = parse(option_env!("MIRAGE_DEBUG_MAX_FIRMWARE_EXITS"));
