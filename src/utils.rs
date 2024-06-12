//! Utils
//!
//! This module expose utilities used accross Mirage, such as types or helper functions.

use core::marker::PhantomData;

/// A marker type that is not Send and not Sync (!Send, !Sync).
pub type PhantomNotSendNotSync = PhantomData<*const ()>;
