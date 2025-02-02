use crate::arch::{mie, misa, mstatus, Mode};
use crate::virt::VirtContext;

/// Generates a precondition for the formal verification and tests it in Miralis
///
/// This macro applies a precondition in the formal verification of Miralis with Kani.
/// This implies that only a subset of the symbolic state is being tested.
/// Therefore, it is crucial to detect any violation of those in Miralis.
/// In the case the precondition is not satisfied, the assertion fails with an explanation.
macro_rules! precondition {
    ($left:expr, $right:expr, $msg:expr) => {{
        #[cfg(kani)]
        {
            $left = $right;
        }
        #[cfg(not(kani))]
        {
            assert_eq!($left, $right, $msg)
        }
    }};
}

/// Applies precondition for Kani and verify they hold on Miralis in Debug mode
pub fn apply_or_verify_preconditions(ctx: &mut VirtContext) {
    let xlen = 0b10; // X-len fixed at 64 bits

    precondition!(ctx.mode, Mode::M, "Mode must be m");
    precondition!(ctx.pc, ctx.pc % (usize::MAX - 4), "PC is equal usize::MAX");
    precondition!(ctx.csr.mie, ctx.csr.mie & mie::ALL_INT, "mie");
    precondition!(ctx.csr.mtvec, ctx.csr.mtvec & !0b10, "mtvec");
    // TODO: opensbi test fails
    // precondition!(ctx.csr.mepc, ctx.csr.mepc & !0b11, "mepc");
    // TODO: csr-ops fails
    /*precondition!(
        ctx.csr.mstatus,
        ctx.csr.mstatus & !mstatus::SXL_FILTER & !mstatus::UXL_FILTER
            | (xlen << mstatus::SXL_OFFSET)
            | (xlen << mstatus::UXL_OFFSET),
        "mstatus"
    );*/
    assert_ne!(
        (ctx.csr.mstatus >> mstatus::MPP_OFFSET) & 0b11,
        0b10,
        "mstatus mpp"
    );
    // TODO: rustdbi test fails
    // precondition!(ctx.csr.sepc, ctx.csr.sepc & !0b11, "sepc");
    // TODO: rustsbi test fails
    /*precondition!(
        ctx.csr.mideleg,
        (ctx.csr.mideleg & mie::ALL_INT) | mie::MIDELEG_READ_ONLY_ONE,
        "mideleg"
    );*/

    // Lock mode is not supported at the moment in Miralis
    // TODO: Zephyr test fails
    /*for i in 0..8 {
        for j in 0..8 {
            precondition!(
                ctx.csr.pmpcfg[i],
                ctx.csr.pmpcfg[i] & !(1 << (7 + j * 8)),
                "pmpcfg"
            );
        }
    }*/

    // We don't have compressed instructions in Miralis
    precondition!(ctx.csr.misa, ctx.csr.misa & !misa::DISABLED, "misa C");

    // We don't have the userspace interrupt delegation in Miralis
    precondition!(ctx.csr.misa, ctx.csr.misa & !misa::N, "misa N");

    // We fix the architecture type to 64 bits
    precondition!(
        ctx.csr.misa,
        (xlen << 62) | (ctx.csr.misa & ((1 << 62) - 1)),
        "misa 64"
    );

    // We must have support for U and S modes in Miralis
    precondition!(ctx.csr.misa, ctx.csr.misa | misa::U | misa::S, "misa U & S");

    // x0 is hardwired to zero
    precondition!(ctx.regs[0], 0, "regs_0");

    // We don't delegate any interrupts in the formal verification
    // TODO: csr-ops fails
    // precondition!(ctx.csr.mideleg, 0, "mideleg");
}
