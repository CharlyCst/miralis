#![no_std]

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Helper macro to check is boolean choice is enabled by the configuration, defaulting to yes.
///
/// The current implementation works around the limitation of const functions in rust at the
/// time of writing.
#[macro_export]
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

pub const fn parse_usize(env_var: Option<&str>) -> Option<usize> {
    match env_var {
        Some(value) => match usize::from_str_radix(value, 10) {
            Ok(value) => Some(value),
            Err(_) => panic!("Failed to parse integed from configuration"),
        },
        None => None,
    }
}

/// Split a string of comma (",") separated values into a list of strings slices.
pub const fn parse_str_list<const LEN: usize>(env_var: Option<&str>) -> [&str; LEN] {
    // First we unwrap the option
    let env_var = match env_var {
        Some(var) => var,
        None => return [""; LEN],
    };

    // Then we iterate over the bytes of the string
    let bytes = env_var.as_bytes();
    let mut res: [&str; LEN] = [""; LEN];
    let mut idx_start = 0;
    let mut idx_curr = 0;
    let mut i = 0;

    while i < LEN {
        // We continue until we find a "," delimiter
        while idx_curr < bytes.len() && bytes[idx_curr] != b',' {
            idx_curr += 1;
        }

        // Then we need to get a sub-slice
        // Unfortunately indexing with a range (`my_str[a..b]`) is not possible
        // in const contexts yet, but splitting at a given index is. So we split
        // it twice to get a sub-slice.
        let sub_slice = &bytes.split_at(idx_start).1;
        let sub_slice = &sub_slice.split_at(idx_curr - idx_start).0;

        // We convert it back into a string, and unwrap it (again, `.unwrap()`
        // is not available).
        let sub_str = match core::str::from_utf8(sub_slice) {
            Ok(sub_str) => sub_str,
            Err(_) => panic!("Invalid string list in configuration"),
        };
        res[i] = sub_str;

        // And we move on to the next element
        idx_curr += 1;
        idx_start = idx_curr;
        i += 1;
    }

    res
}

/// Returns the len of a list of comma (",") separated values.
pub const fn str_list_len(env_var: Option<&str>) -> usize {
    // First we unwrap the option
    let env_var = match env_var {
        Some(var) => var,
        None => return 0,
    };

    let mut len = 1;
    let mut i = 0;
    let bytes = env_var.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b',' {
            len += 1;
        }
        i += 1;
    }
    len
}

pub const fn parse_str_or(env_var: Option<&'static str>, default: &'static str) -> &'static str {
    match env_var {
        Some(var) => var,
        None => default,
    }
}

pub const fn parse_usize_or(env_var: Option<&str>, default: usize) -> usize {
    match parse_usize(env_var) {
        Some(value) => value,
        None => default,
    }
}
