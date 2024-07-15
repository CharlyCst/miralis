//! Utils
//!
//! This module expose utilities used accross Mirage, such as types or helper functions.

use core::marker::PhantomData;

/// A marker type that is not Send and not Sync (!Send, !Sync).
pub type PhantomNotSendNotSync = PhantomData<*const ()>;

// Can i place it here?
pub fn calculate_offset(self_value: usize, imm: isize) -> usize {
    if imm >= 0 {
        self_value + (imm) as usize
    } else {
        self_value - (-imm) as usize
    }
}
