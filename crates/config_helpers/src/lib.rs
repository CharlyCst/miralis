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
// Required to parse environment variables at compile time.
// Can be removed once usize::from_str_radix stabilized as const, hopefully soon.
// See https://github.com/rust-lang/rust/pull/124941
//
// Source (and license), adapted for Miralis:
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

pub const fn parse_usize(env_var: Option<&str>) -> Option<usize> {
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

pub const fn parse_usize_or(env_var: Option<&str>, default: usize) -> usize {
    match parse_usize(env_var) {
        Some(value) => value,
        None => default,
    }
}
