//! Utils
//!
//! This module exposes utilities used across Miralis, such as types or helper functions.

use core::marker::PhantomData;

use crate::arch::Width;

/// A marker type that is not Send and not Sync (!Send, !Sync).
pub type PhantomNotSendNotSync = PhantomData<*const ()>;

/// Compute the address from a base address plus an immediate.
pub fn calculate_addr(self_value: usize, imm: isize) -> usize {
    if imm >= 0 {
        self_value + (imm) as usize
    } else {
        self_value - (-imm) as usize
    }
}

/// Performs a sign extension assuming the provided width of the value.
pub fn sign_extend(value: usize, width: Width) -> usize {
    match width {
        Width::Byte => value as u8 as i8 as isize as usize,
        Width::Byte2 => value as u16 as i16 as isize as usize,
        Width::Byte4 => value as u32 as i32 as isize as usize,
        Width::Byte8 => value, // Already 64 bits, nothing to do
    }
}

/// Extracts the bitwise representation and set to the corresponding signed value
pub fn bits_to_int(raw: usize, start_bit: isize, end_bit: isize) -> isize {
    let mask = (1 << (end_bit - start_bit + 1)) - 1;
    let value = (raw >> start_bit) & mask;

    // Check if the most significant bit is set (indicating a negative value)
    if value & (1 << (end_bit - start_bit)) != 0 {
        // Extend the sign bit to the left
        let sign_extension = !0 << (end_bit - start_bit);
        value as isize | sign_extension
    } else {
        value as isize
    }
}

/// Compare two &str, valid in compile time contexts.
///
/// The equality operator on &str is not const yet, therefore we need to implement a const function
/// that compare strings manually to work around that limitation.
pub const fn const_str_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    // We are going to do the comparison byte by byte
    let a = a.as_bytes();
    let b = b.as_bytes();

    // We use a while loop as for loops are not yet stable in const contexts
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_extension() {
        // 1 byte
        assert_eq!(sign_extend(0xf0, Width::Byte), 0xfffffffffffffff0);
        assert_eq!(sign_extend(0x80, Width::Byte), 0xffffffffffffff80);
        assert_eq!(sign_extend(0x7f, Width::Byte), 0x7f);
        assert_eq!(sign_extend(0x00, Width::Byte), 0x00);

        // 2 bytes
        assert_eq!(sign_extend(0xf000, Width::Byte2), 0xfffffffffffff000);
        assert_eq!(sign_extend(0x8000, Width::Byte2), 0xffffffffffff8000);
        assert_eq!(sign_extend(0x7fff, Width::Byte2), 0x7fff);
        assert_eq!(sign_extend(0x0000, Width::Byte2), 0x0000);
        assert_eq!(sign_extend(0x00ff, Width::Byte2), 0x00ff);

        // 4 bytes
        assert_eq!(sign_extend(0xf0000000, Width::Byte4), 0xfffffffff0000000);
        assert_eq!(sign_extend(0x80000000, Width::Byte4), 0xffffffff80000000);
        assert_eq!(sign_extend(0x7fffffff, Width::Byte4), 0x7fffffff);
        assert_eq!(sign_extend(0x00000000, Width::Byte4), 0x00000000);
        assert_eq!(sign_extend(0x0000ffff, Width::Byte4), 0x0000ffff);
    }

    #[test]
    fn str_eq() {
        assert!(const_str_eq("foo", "foo"));
        assert!(const_str_eq("", ""));
        assert!(!const_str_eq("foo", "fooo"));
        assert!(!const_str_eq("fooo", "foo"));
        assert!(!const_str_eq("bar", "foo"));

        // Also check that the function is const
        assert!(const { const_str_eq("foo", "foo") });
    }
}
