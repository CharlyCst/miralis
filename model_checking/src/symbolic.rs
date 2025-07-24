//! Symbolic Context and Values
//!
//! This modules is responsible for instantiating symbolic values, individually or as a whole
//! context. We make sure that this module can compile and be tested even without Kani installed,
//! in which case concrete values are used in place of symbolic ones.

use miralis::arch::{menvcfg, mie, misa, mstatus, Arch, Architecture, ExtensionsCapability, Mode};
use miralis::host::MiralisContext;
use miralis::platform::{Plat, Platform};
use miralis::virt::VirtContext;
use softcore_rv64::Core;

// use sail_model::SailVirtCtx;
use crate::adapters;

/// A dummy size for Miralis, used to initialize the PMP for symbolic execution.
pub const MIRALIS_SIZE: usize = 0x1000;

/// Generates an arbitrary value.
///
/// A type can be provided as argument, otherwise it will be inferred if possible.
/// A default value can be provided in addition to the type, it will be used during concrete
/// execution.
///
/// This macro either generate a value of a type, or an arbitrary Kani value during model checking.
/// We use this macro to make our Kani proofs runnable as simple tests, which ensures that we don't
/// break the Kani verification harnesses.
macro_rules! any {
    () => {{
        #[cfg(kani)]
        {
            kani::any()
        }
        #[cfg(all(not(kani), not(feature = "rand")))]
        {
            Default::default()
        }
        #[cfg(all(not(kani), feature = "rand"))]
        {
            rand::random()
        }
    }};
    ($t:ty) => {{
        #[cfg(kani)]
        {
            kani::any::<$t>()
        }
        #[cfg(all(not(kani), not(feature = "rand")))]
        {
            <$t>::default()
        }
        #[cfg(all(not(kani), feature = "rand"))]
        {
            rand::random::<$t>()
        }
    }};
    ($t:ty, $value:tt) => {{
        #[cfg(kani)]
        {
            kani::any::<$t>()
        }
        #[cfg(not(kani))]
        {
            let val: $t = $value;
            val
        }
    }};
}

/// Return a new context with symbolic values
pub fn new_ctx(available_extension: ExtensionsCapability) -> VirtContext {
    let mut ctx = VirtContext::new(0, 0, available_extension);

    // Mode
    ctx.mode = Mode::M;

    // We don't want overflows here
    ctx.pc = any!(usize) % (usize::MAX - 4);
    ctx.nb_pmp = 16;
    ctx.pmp_grain = 10;

    // Pick a previous privilege mode
    let mpp = match any!(u8) % 3 {
        0 => 0b00,
        1 => 0b01,
        2 => 0b11,
        _ => unreachable!(),
    };
    let xlen = 0b10; // X-len fixed at 64 bits

    // Add other csr
    ctx.hart_id = any!();
    ctx.csr.misa = any!();
    ctx.csr.mie = any!(usize) & mie::ALL_INT;
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
    ctx.csr.menvcfg = any!(usize) & (menvcfg::FIOM_FILTER | menvcfg::STCE_FILTER);
    // ctx.csr.mseccfg = any!();
    ctx.csr.mcause = any!();
    ctx.csr.mepc = any!(usize) & (!0b11);
    ctx.csr.mtval = any!();
    // ctx.csr.mtval2 = any!(); - TODO: What should we do with this?
    ctx.csr.mstatus = any!(usize)
        & (mstatus::SD_FILTER
            | mstatus::TSR_FILTER
            | mstatus::TW_FILTER
            | mstatus::TVM_FILTER
            | mstatus::MXR_FILTER
            | mstatus::SUM_FILTER
            | mstatus::MPRV_FILTER
            | mstatus::FS_FILTER
            | mstatus::VS_FILTER
            | mstatus::SPP_FILTER
            | mstatus::MPIE_FILTER
            | mstatus::SPIE_FILTER
            | mstatus::MIE_FILTER
            | mstatus::SIE_FILTER);
    ctx.csr.mstatus = ctx.csr.mstatus
        | (xlen << mstatus::SXL_OFFSET)
        | (xlen << mstatus::UXL_OFFSET)
        | (mpp << mstatus::MPP_OFFSET);
    // ctx.csr.mtinst = any!();
    ctx.csr.mconfigptr = any!();
    // ctx.csr.stvec = any!();
    ctx.csr.scounteren = any!();
    ctx.csr.senvcfg = any!(usize) & menvcfg::FIOM_FILTER;
    ctx.csr.sscratch = any!();
    ctx.csr.sepc = any!(usize) & (!0b11);
    ctx.csr.scause = any!();
    ctx.csr.stval = any!();
    ctx.csr.satp = any!();
    //ctx.csr.scontext = any!(); // TODO: What should we do with this?
    ctx.csr.medeleg = any!();
    ctx.csr.mideleg = (any!(usize) & mie::ALL_INT) | mie::MIDELEG_READ_ONLY_ONE;
    ctx.csr.pmpcfg = [any!(); 8];
    ctx.csr.pmpaddr = [any!(usize) >> 10; 64]; // encodes bits [56:2] of the 56 bits address space

    // ctx.csr.mhpmcounter = [any!(); 29]; todo: What should we do?
    // ctx.csr.mhpmevent = [any!(); 29]; todo: What should we do?

    // Lock mode is not supported at the moment in Miralis
    for i in 0..8 {
        for j in 0..8 {
            let offset = j * 8;
            let mut pmpcfg = ctx.csr.pmpcfg[i];

            // Lock mode not supported
            pmpcfg &= !(1 << (7 + offset));

            // NA4 not supported for PMP grain >= 1
            // If bit 4 is 1, then either NA4 or NAPOT is selected.
            // In that case, we set bit 3, which forces NAPOT.
            let nax = pmpcfg & (0b00010000 << offset);
            pmpcfg |= nax >> 1;

            // If R = 0 and W = 1, then we set RWX to 0
            // This is what the Sail spec does too.
            if pmpcfg & (0b11 << offset) == 0b10 {
                pmpcfg &= !(0b111 << offset)
            }

            ctx.csr.pmpcfg[i] = pmpcfg;
        }
    }

    // Zero-out unsupported PMP registers
    for i in ctx.nb_pmp..64 {
        ctx.csr.pmpaddr[i] = 0;
    }

    // We don't have compressed instructions in Miralis
    ctx.csr.misa &= !misa::DISABLED;

    // We don't have the userspace interrupt delegation in Miralis
    ctx.csr.misa &= !misa::N;

    // The model does not support H mode
    ctx.csr.misa &= !misa::H;

    // We fix the architecture type to 64 bits
    ctx.csr.misa = (xlen << 62) | (ctx.csr.misa & ((1 << 62) - 1));

    // We must have support for U and S modes in Miralis
    ctx.csr.misa |= misa::U;
    ctx.csr.misa |= misa::S;

    // Now we allocate general purpose registers
    ctx.regs = [any!(usize); 32];
    // x0 is hardwired to zero
    ctx.regs[0] = 0;

    // We don't delegate any interrupts in the formal verification
    ctx.csr.mideleg = 0;

    ctx
}

/// Return a Miralis and Sail virtual context filled with symbolic values.
///
/// A [MiralisContext] containing concrete value is also returned, it is required for emulation by
/// Miralis and mostly containst the list of hardware extensions (which are fixed during model
/// checking).
pub fn new_symbolic_contexts() -> (VirtContext, MiralisContext, Core) {
    // Initialize Miralis's own context
    let hw = unsafe { Arch::detect_hardware() };
    let mctx = MiralisContext::new(hw, Plat::get_miralis_start(), MIRALIS_SIZE);

    // We first create a symbolic context
    let ctx = new_ctx(mctx.hw.extensions.clone());
    // Then we copy the symbolic values into a Sail context
    let core = adapters::miralis_to_rv_core(&ctx);

    (ctx, mctx, core)
}
