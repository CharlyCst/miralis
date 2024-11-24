use crate::arch::{mstatus, parse_mpp_return_mode, Arch, Architecture, Csr, Mode, Register};
use crate::host::MiralisContext;
use crate::virt::{
    HwRegisterContextSetter, RegisterContextGetter, RegisterContextSetter, VirtContext, VirtCsr,
};

pub fn emulate_wfi(ctx: &mut VirtContext, _mctx: &mut MiralisContext) {
    // NOTE: for now there is no safeguard which guarantees that we will eventually get
    // an interrupt, so the firmware might be able to put the core in perpetual sleep
    // state.

    // Set mie to csr.mie, even if mstatus.MIE bit is cleared.
    unsafe {
        Arch::write_csr(Csr::Mie, ctx.csr.mie);
    }

    Arch::wfi();
    ctx.pc += 4;
}

pub fn emulate_csrrw(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    rs1: &Register,
) {
    let tmp = ctx.get(csr);
    ctx.set_csr(csr, ctx.get(rs1), mctx);
    ctx.set(rd, tmp);
    ctx.pc += 4;
}

pub fn emulate_csrrs(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    rs1: &Register,
) {
    let tmp = ctx.get(csr);
    ctx.set_csr(csr, tmp | ctx.get(rs1), mctx);
    ctx.set(rd, tmp);
    ctx.pc += 4;
}

pub fn emulate_csrrwi(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    uimm: &usize,
) {
    ctx.set(rd, ctx.get(csr));
    ctx.set_csr(csr, *uimm, mctx);
    ctx.pc += 4;
}

pub fn emulate_csrrsi(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    uimm: &usize,
) {
    let tmp = ctx.get(csr);
    ctx.set_csr(csr, tmp | uimm, mctx);
    ctx.set(rd, tmp);
    ctx.pc += 4;
}

pub fn emulate_csrrc(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    rs1: &Register,
) {
    let tmp = ctx.get(csr);
    ctx.set_csr(csr, tmp & !ctx.get(rs1), mctx);
    ctx.set(rd, tmp);
    ctx.pc += 4;
}

pub fn emulate_csrrci(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    csr: &Csr,
    rd: &Register,
    uimm: &usize,
) {
    let tmp = ctx.get(csr);
    ctx.set_csr(csr, tmp & !uimm, mctx);
    ctx.set(rd, tmp);
    ctx.pc += 4;
}

pub fn emulate_mret(ctx: &mut VirtContext, mctx: &mut MiralisContext) {
    match parse_mpp_return_mode(ctx.csr.mstatus) {
        Mode::M => {
            log::trace!("mret to m-mode to {:x}", ctx.trap_info.mepc);
            // Mret is jumping back to machine mode, do nothing
        }
        Mode::S if mctx.hw.extensions.has_s_extension => {
            log::trace!("mret to s-mode with MPP to {:x}", ctx.trap_info.mepc);
            // Mret is jumping to supervisor mode, the runner is the guest OS
            ctx.mode = Mode::S;

            VirtCsr::set_csr_field(
                &mut ctx.csr.mstatus,
                mstatus::MPRV_OFFSET,
                mstatus::MPRV_FILTER,
                0,
            );
        }
        Mode::U => {
            log::trace!("mret to u-mode with MPP");
            // Mret is jumping to user mode, the runner is the guest OS
            ctx.mode = Mode::U;

            VirtCsr::set_csr_field(
                &mut ctx.csr.mstatus,
                mstatus::MPRV_OFFSET,
                mstatus::MPRV_FILTER,
                0,
            );
        }
        _ => {
            panic!(
                "MRET is not going to M/S/U mode: {} with MPP {:x}",
                ctx.csr.mstatus,
                (ctx.csr.mstatus & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET
            );
        }
    }
    // Modify mstatus
    // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
    if false {
        VirtCsr::set_csr_field(
            &mut ctx.csr.mstatus,
            mstatus::MPV_OFFSET,
            mstatus::MPV_FILTER,
            0,
        );
    }

    // MIE = MPIE, MPIE = 1, MPRV = 0
    let mpie = (ctx.csr.mstatus & mstatus::MPIE_FILTER) >> mstatus::MPIE_OFFSET;

    VirtCsr::set_csr_field(
        &mut ctx.csr.mstatus,
        mstatus::MPIE_OFFSET,
        mstatus::MPIE_FILTER,
        1,
    );
    VirtCsr::set_csr_field(
        &mut ctx.csr.mstatus,
        mstatus::MIE_OFFSET,
        mstatus::MIE_FILTER,
        mpie,
    );
    VirtCsr::set_csr_field(
        &mut ctx.csr.mstatus,
        mstatus::MPP_OFFSET,
        mstatus::MPP_FILTER,
        0,
    );

    // Jump back to firmware
    ctx.pc = ctx.csr.mepc;
}

pub fn emulate_sfence_vma(
    ctx: &mut VirtContext,
    _mctx: &mut MiralisContext,
    rs1: &Register,
    rs2: &Register,
) {
    unsafe {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        Arch::sfencevma(vaddr, asid);
        ctx.pc += 4;
    }
}

pub fn emulate_hfence_gvma(
    ctx: &mut VirtContext,
    _mctx: &mut MiralisContext,
    rs1: &Register,
    rs2: &Register,
) {
    unsafe {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        Arch::hfencegvma(vaddr, asid);
        ctx.pc += 4;
    }
}

pub fn emulate_hfence_vvma(
    ctx: &mut VirtContext,
    _mctx: &mut MiralisContext,
    rs1: &Register,
    rs2: &Register,
) {
    unsafe {
        let vaddr = match rs1 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        let asid = match rs2 {
            Register::X0 => None,
            reg => Some(ctx.get(reg)),
        };
        Arch::hfencevvma(vaddr, asid);
        ctx.pc += 4;
    }
}
