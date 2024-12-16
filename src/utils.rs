//! Utils
//!
//! This module expose utilities used accross Miralis, such as types or helper functions.

use core::marker::PhantomData;

use crate::arch::{Csr, Register, Width};
use crate::virt::traits::RegisterContextGetter;
use crate::virt::VirtContext;

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

// —————————————————————————————— Debug Helper —————————————————————————————— //

/// Log the current context using the trace log level.
pub fn log_ctx(ctx: &VirtContext) {
    let trap_info = &ctx.trap_info;
    log::trace!(
        "Trapped on hart {}:  {:?}",
        ctx.hart_id,
        ctx.trap_info.get_cause()
    );
    log::trace!(
        "  mstatus: 0x{:<16x} mepc: 0x{:x}",
        trap_info.mstatus,
        trap_info.mepc
    );
    log::trace!(
        "  mtval:   0x{:<16x} exits: {}  {:?}-mode",
        ctx.trap_info.mtval,
        ctx.nb_exits,
        ctx.mode
    );
    log::trace!(
        "  x1  {:<16x}  x2  {:<16x}  x3  {:<16x}",
        ctx.get(Register::X1),
        ctx.get(Register::X2),
        ctx.get(Register::X3)
    );
    log::trace!(
        "  x4  {:<16x}  x5  {:<16x}  x6  {:<16x}",
        ctx.get(Register::X4),
        ctx.get(Register::X5),
        ctx.get(Register::X6)
    );
    log::trace!(
        "  x7  {:<16x}  x8  {:<16x}  x9  {:<16x}",
        ctx.get(Register::X7),
        ctx.get(Register::X8),
        ctx.get(Register::X9)
    );
    log::trace!(
        "  x10 {:<16x}  x11 {:<16x}  x12 {:<16x}",
        ctx.get(Register::X10),
        ctx.get(Register::X11),
        ctx.get(Register::X12)
    );
    log::trace!(
        "  x13 {:<16x}  x14 {:<16x}  x15 {:<16x}",
        ctx.get(Register::X13),
        ctx.get(Register::X14),
        ctx.get(Register::X15)
    );
    log::trace!(
        "  x16 {:<16x}  x17 {:<16x}  x18 {:<16x}",
        ctx.get(Register::X16),
        ctx.get(Register::X17),
        ctx.get(Register::X18)
    );
    log::trace!(
        "  x19 {:<16x}  x20 {:<16x}  x21 {:<16x}",
        ctx.get(Register::X19),
        ctx.get(Register::X20),
        ctx.get(Register::X21)
    );
    log::trace!(
        "  x22 {:<16x}  x23 {:<16x}  x24 {:<16x}",
        ctx.get(Register::X22),
        ctx.get(Register::X23),
        ctx.get(Register::X24)
    );
    log::trace!(
        "  x25 {:<16x}  x26 {:<16x}  x27 {:<16x}",
        ctx.get(Register::X25),
        ctx.get(Register::X26),
        ctx.get(Register::X27)
    );
    log::trace!(
        "  x28 {:<16x}  x29 {:<16x}  x30 {:<16x}",
        ctx.get(Register::X28),
        ctx.get(Register::X29),
        ctx.get(Register::X30)
    );
    log::trace!(
        "  x31 {:<16x}  mie {:<16x}  mip {:<16x}",
        ctx.get(Register::X31),
        ctx.get(Csr::Mie),
        ctx.get(Csr::Mip)
    );
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
