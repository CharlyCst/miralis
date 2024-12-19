//! Symbolic Context and Values
//!
//! This modules is responsible for instantiating symbolic values, individually or as a whole
//! context. We make sure that this module can compile and be tested even without Kani installed,
//! in which case concrete values are used in place of symbolic ones.

use miralis::arch::{misa, mstatus, Arch, Architecture, ExtensionsCapability, Mode};
use miralis::host::MiralisContext;
use miralis::platform::{Plat, Platform};
use miralis::virt::VirtContext;
use sail_model::SailVirtCtx;

use crate::adapters;

/// Generates an arbitrary value.
///
/// A type can be provided as argument, otherwise it will be inferred if possible.
///
/// This macro either generate the default value of a type, or an arbitrary Kani value during model
/// checking. We use this macro to make our Kani proofs runnable as simple tests, which ensures
/// that we don't break the Kani verification harnesses.
macro_rules! any {
    () => {{
        #[cfg(kani)]
        {
            kani::any()
        }
        #[cfg(not(kani))]
        {
            Default::default()
        }
    }};
    ($t:ty) => {{
        #[cfg(kani)]
        {
            kani::any::<$t>()
        }
        #[cfg(not(kani))]
        {
            <$t>::default()
        }
    }};
}

/// Return a new context with symbolic values
pub fn new_ctx() -> VirtContext {
    let mut ctx = VirtContext::new(
        0,
        0,
        ExtensionsCapability {
            has_crypto_extension: true,
            has_sstc_extension: false,
            is_sstc_enabled: false,
            has_zicntr: true,
            has_h_extension: false,
            has_s_extension: false,
            has_v_extension: true,
            has_zihpm_extension: true,
        },
    );

    // Mode
    ctx.mode = Mode::M;
    ctx.pc = any!();
    ctx.nb_pmp = 16;

    // Pick a previous privilege mode
    let mpp = match any!(u8) % 3 {
        0 => 0b00,
        1 => 0b01,
        2 => 0b11,
        _ => unreachable!(),
    };

    // Add other csr
    ctx.hart_id = any!();
    ctx.csr.misa = any!();
    ctx.csr.mie = any!();
    ctx.csr.mip = any!();
    ctx.csr.mtvec = any!(usize) & !0b10; // 10 is  an illegal trap vector
    ctx.csr.mscratch = any!();
    ctx.csr.mvendorid = any!();
    ctx.csr.marchid = any!();
    // ctx.csr.mimpid = any!();
    ctx.csr.mcycle = any!();
    ctx.csr.minstret = any!();
    ctx.csr.mcountinhibit = any!();
    ctx.csr.mcounteren = any!();
    ctx.csr.menvcfg = any!();
    // ctx.csr.mseccfg = any!();
    ctx.csr.mcause = any!();
    ctx.csr.mepc = any!(usize) & (!0b11);
    ctx.csr.mtval = any!();
    // ctx.csr.mtval2 = any!(); - TODO: What should we do with this?
    ctx.csr.mstatus = any!(usize) & !(0b11 << mstatus::MPP_OFFSET) | (mpp << mstatus::MPP_OFFSET);
    // ctx.csr.mtinst = any!();
    ctx.csr.mconfigptr = any!();
    // ctx.csr.stvec = any!();
    ctx.csr.scounteren = any!();
    ctx.csr.senvcfg = any!();
    ctx.csr.sscratch = any!();
    ctx.csr.sepc = any!();
    ctx.csr.scause = any!();
    ctx.csr.stval = any!();
    ctx.csr.satp = any!();
    //ctx.csr.scontext = any!(); // TODO: What should we do with this?
    ctx.csr.medeleg = any!();
    ctx.csr.mideleg = any!();
    ctx.csr.pmpcfg = [any!(); 8];
    ctx.csr.pmpaddr = [any!(); 64];
    // ctx.csr.mhpmcounter = [any!(); 29]; todo: What should we do?
    // ctx.csr.mhpmevent = [any!(); 29]; todo: What should we do?

    // Lock mode is not supported at the moment in Miralis
    for i in 0..8 {
        for j in 0..8 {
            ctx.csr.pmpcfg[i] &= !(1 << (7 + j * 8));
        }
    }

    // We don't have compressed instructions in Miralis
    ctx.csr.misa &= !misa::DISABLED;

    // We don't have the userspace interrupt delegation in Miralis
    ctx.csr.misa &= !misa::N;

    // We fix the architecture type to 64 bits
    ctx.csr.misa = (0b10 << 62) | (ctx.csr.misa & ((1 << 62) - 1));

    // We must have support for usermode in Miralis
    ctx.csr.misa |= misa::U;

    ctx
}

/// Return a Miralis and Sail virtual context filled with symbolic values.
///
/// A [MiralisContext] containing concrete value is also returned, it is required for emulation by
/// Miralis and mostly containst the list of hardware extensions (which are fixed during model
/// checking).
pub fn new_symbolic_contexts() -> (VirtContext, MiralisContext, SailVirtCtx) {
    // We first create a symbolic context
    let ctx = new_ctx();
    // Then we copy the symbolic values into a Sail context
    let sail_ctx = adapters::miralis_to_sail(&ctx);

    // Initialize Miralis's own context
    let hw = unsafe { Arch::detect_hardware() };
    let mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

    (ctx, mctx, sail_ctx)
}
