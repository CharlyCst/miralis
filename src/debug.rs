//! Debug utils for Miralis

use crate::arch::{Arch, Architecture, Csr};
use crate::config::TARGET_STACK_SIZE;

// ————————————————————————————— Logging Utils —————————————————————————————— //

/// Emit a warning only once.
///
/// This macro calls log::warn internally and forwards all arguments.
macro_rules! warn_once {
    ($($args:tt)*) => {{
        use core::sync::atomic::{AtomicBool, Ordering};
        static IS_FIRST_WARN: AtomicBool = AtomicBool::new(true);

        if IS_FIRST_WARN.swap(false, Ordering::Relaxed) == true {
            log::warn!($($args)*);
        }
    }}
}

pub(crate) use warn_once;

// ————————————————————————————— Unimplemented —————————————————————————————— //

/// Mark a code path as not yet implemented, causing a runtime panic.
///
/// This macro si equivalent to the standard [unimplemented!], except it makes model checking with
/// Kani succeed when reached rather than fail.
///
/// See [Kani discussion](https://github.com/model-checking/kani/discussions/3746) for background.
macro_rules! unimplemented {
   ($($arg:tt)*) => {
       #[cfg(kani)]
       kani::assume(false);
       unimplemented!($($arg)*);
    };
}

pub(crate) use unimplemented;

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
/// # Safety
///
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
/// # Safety
///
/// This function assumes a single-core system for now.
pub unsafe fn log_stack_usage(stack_start: usize) {
    /// Percent usage threshold for emitting a warning.
    const WARNING_THRESHOLD: usize = 80;

    // Get stack usage
    let stack_bottom = stack_start;
    let hart_id = Arch::read_csr(Csr::Mhartid);
    let stack_bottom = stack_bottom + hart_id * TARGET_STACK_SIZE;
    let stack_top = stack_bottom + TARGET_STACK_SIZE;
    let max_stack_usage = get_max_stack_usage(stack_top, stack_bottom);

    // Compute percentage with one 1 decimal precision
    let permil = (1000 * max_stack_usage + TARGET_STACK_SIZE / 2) / TARGET_STACK_SIZE;
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
