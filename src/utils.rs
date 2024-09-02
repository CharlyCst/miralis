//! Utils
//!
//! This module expose utilities used accross Miralis, such as types or helper functions.

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

/// Performws a sign extension assuming the provided width of the value.
pub fn sign_extend(value: usize, width: Width) -> usize {
    match width {
        Width::Byte => value as u8 as i8 as isize as usize,
        Width::Byte2 => value as u16 as i16 as isize as usize,
        Width::Byte4 => value as u32 as i32 as isize as usize,
        Width::Byte8 => value, // Already 64 bits, nothing to do
    }
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
}
