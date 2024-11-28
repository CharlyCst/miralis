use miralis::arch::{mstatus, Arch, Architecture, Mode};
use miralis::host::MiralisContext;
use miralis::platform::{Plat, Platform};

#[macro_use]
mod symbolic;
mod adapters;

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn mret() {
    let mpp = match any!(u8) % 3 {
        0 => 0b00,
        1 => 0b01,
        2 => 0b11,
        _ => unreachable!(),
    };
    let mstatus = any!(usize) & !(0b11 << mstatus::MPP_OFFSET);

    let mut ctx = symbolic::new_ctx();

    ctx.csr.mepc = any!(usize) & (!0b11);
    ctx.csr.sepc = any!();
    ctx.csr.mstatus = mstatus | (mpp << mstatus::MPP_OFFSET);
    ctx.mode = Mode::M;
    ctx.pc = any!();

    let mut sail_ctx = adapters::miralis_to_sail(&ctx);
    sail_model::execute_MRET(&mut sail_ctx);

    // Initialize Miralis's own context
    let hw = unsafe { Arch::detect_hardware() };
    let mut mctx = MiralisContext::new(hw, Plat::get_miralis_start(), 0x1000);

    ctx.emulate_mret(&mut mctx);

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "mret instruction emulation is not correct"
    );
}
