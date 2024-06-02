//! Debug utils for Mirage

use crate::{_stack_bottom, _stack_top, config};

// ——————————————————————————— Max Firmware Exits ——————————————————————————— //

/// Returns the maximum number of firmware exits before panicking.
///
/// If the value returned is None, there is no maximum cap.
///
/// NOTE: For now we still need some basic runtime parsing, but once
/// https://github.com/rust-lang/rust/pull/99322 gets merged we can convert this function to a
/// constant.
pub fn get_max_firmware_exits() -> Option<usize> {
    match config::MAX_FIRMWARE_EXIT {
        Some(env_var) => usize::from_str_radix(env_var, 10).ok(),
        None => None,
    }
}

// ———————————————————————————— Max Stack Usage ————————————————————————————— //

/// A well known memory pattern
///
/// This pattern can be used to fill unitialized memory, which might be useful for a variety of
/// debug purpose.
const MEMORY_PATTERN: u32 = 0x0BADBED0;

/// Returns the maximum stack usage
///
/// This function traverses the stack to check how much of the stack has been used. This relies on
/// the stack being initialized with the proper pattern.
///
/// # SAFETY:
/// This function requires stack_top and stack_bottom to point to the start and end of the stack,
/// and that the stack is not mutated for the whole duration of the function.
unsafe fn get_max_stack_usage(stack_top: usize, stack_bottom: usize) -> usize {
    const PATTERN_SIZE: usize = core::mem::size_of::<u32>();

    assert!(stack_bottom < stack_top);
    assert!(stack_bottom % 4 == 0);
    assert!(stack_top % 4 == 0);

    let stack_ptr = stack_bottom as *const u32;
    let len = (stack_top - stack_bottom) / PATTERN_SIZE;
    let stack = core::slice::from_raw_parts(stack_ptr, len);

    let mut counter = 0;
    for data in stack.iter() {
        if *data != MEMORY_PATTERN {
            break;
        }
        counter += 1;
    }

    // Return used memory
    (len - counter) * PATTERN_SIZE
}

/// Display debug information related to maximal stack usage
///
/// # SAFETY:
/// This function assumes a single-core system for now.
pub unsafe fn log_stack_usage() {
    /// Percent usage threshold for emitting a warning.
    const WARNING_THRESHOLD: usize = 80;

    // Get stack usage
    let stack_top = (&_stack_top) as *const u8 as usize;
    let stack_bottom = (&_stack_bottom) as *const u8 as usize;
    assert!(stack_top > stack_bottom);
    let stack_size = stack_top - stack_bottom;
    let max_stack_usage = get_max_stack_usage(stack_top, stack_bottom);

    // Compute percentage with one 1 decimal precision
    let permil = (1000 * max_stack_usage + stack_size / 2) / stack_size;
    let percent = permil / 10;
    let decimal = permil % 100;

    // Display stack usage
    if percent == 100 {
        log::error!("Stack overflow: stack size increase required");
    } else if percent > WARNING_THRESHOLD {
        log::warn!(
            "Maximal stack usage: {} bytes ({}.{}%) - consider increasing stack size",
            max_stack_usage,
            percent,
            decimal
        );
    } else {
        log::info!(
            "Maximal stack usage: {} bytes ({}.{}%)",
            max_stack_usage,
            percent,
            decimal
        );
    }
}
