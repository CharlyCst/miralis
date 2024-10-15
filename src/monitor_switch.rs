use crate::ace::core::architecture::control_status_registers::ReadWriteRiscvCsr;
use crate::ace::core::control_data::HardwareHart;
use crate::virt::VirtContext;

pub fn overwrite_hardware_hart_with_virtctx(hw: &mut HardwareHart, ctx: &mut VirtContext) {
    // Save normal registers
    for i in 0..32 {
        hw.hypervisor_hart.hypervisor_hart_state.gprs.0[i] = ctx.regs[i]
    }

    // Save CSR registers

    // M mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mepc = ReadWriteRiscvCsr(ctx.csr.mepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mcause = ReadWriteRiscvCsr(ctx.csr.mcause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.medeleg = ReadWriteRiscvCsr(ctx.csr.medeleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mideleg = ReadWriteRiscvCsr(ctx.csr.mideleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mie = ReadWriteRiscvCsr(ctx.csr.mie);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.mip = ReadWriteRiscvCsr(ctx.csr.mip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mstatus = ReadWriteRiscvCsr(ctx.csr.mstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtinst = ReadWriteRiscvCsr(ctx.csr.mtinst);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval = ReadWriteRiscvCsr(ctx.csr.mtval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval2 = ReadWriteRiscvCsr(ctx.csr.mtval2);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mtvec = ReadWriteRiscvCsr(ctx.csr.mtvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.mscratch = ReadWriteRiscvCsr(ctx.csr.mscratch);

    // S mode registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sstatus = ReadWriteRiscvCsr(ctx.csr.sstatus);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sie = ReadWriteRiscvCsr(ctx.csr.sie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stvec = ReadWriteRiscvCsr(ctx.csr.stvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scounteren = ReadWriteRiscvCsr(ctx.csr.scounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.senvcfg = ReadWriteRiscvCsr(ctx.csr.senvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sscratch = ReadWriteRiscvCsr(ctx.csr.sscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sepc = ReadWriteRiscvCsr(ctx.csr.sepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scause = ReadWriteRiscvCsr(ctx.csr.scause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stval = ReadWriteRiscvCsr(ctx.csr.stval);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sip = ReadWriteRiscvCsr(ctx.csr.sip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.satp = ReadWriteRiscvCsr(ctx.csr.satp);

    // HS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hstatus = ReadWriteRiscvCsr(ctx.csr.hstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hedeleg = ReadWriteRiscvCsr(ctx.csr.hedeleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hideleg = ReadWriteRiscvCsr(ctx.csr.hideleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hie = ReadWriteRiscvCsr(ctx.csr.hie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hcounteren = ReadWriteRiscvCsr(ctx.csr.hcounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeie = ReadWriteRiscvCsr(ctx.csr.hgeie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htval = ReadWriteRiscvCsr(ctx.csr.htval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hip = ReadWriteRiscvCsr(ctx.csr.hip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hvip = ReadWriteRiscvCsr(ctx.csr.hvip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htinst = ReadWriteRiscvCsr(ctx.csr.htinst);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeip = ReadWriteRiscvCsr(ctx.csr.hgeip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.henvcfg = ReadWriteRiscvCsr(ctx.csr.henvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgatp = ReadWriteRiscvCsr(ctx.csr.hgatp);

    // HS mode debug registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.hcontext = ReadWriteRiscvCsr(ctx.csr.hcontext);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htimedelta = ReadWriteRiscvCsr(ctx.csr.htimedelta);

    // VS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsstatus = ReadWriteRiscvCsr(ctx.csr.vsstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsie = ReadWriteRiscvCsr(ctx.csr.vsie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsip = ReadWriteRiscvCsr(ctx.csr.vsip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstvec = ReadWriteRiscvCsr(ctx.csr.vstvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsscratch = ReadWriteRiscvCsr(ctx.csr.vsscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsepc = ReadWriteRiscvCsr(ctx.csr.vsepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vscause = ReadWriteRiscvCsr(ctx.csr.vscause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstval = ReadWriteRiscvCsr(ctx.csr.vstval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsatp = ReadWriteRiscvCsr(ctx.csr.vsatp);
}

pub fn overwrite_virtctx_with_hardware_hart(ctx: &mut VirtContext,hw: &mut HardwareHart) {
    // Save normal registers
    for i in 0..32 {
        ctx.regs[i] = hw.hypervisor_hart.hypervisor_hart_state.gprs.0[i]
    }

    // Save CSR registers

    // M mode registers
    ctx.csr.mepc = hw.hypervisor_hart.hypervisor_hart_state.csrs.mepc.read();
    ctx.csr.mcause = hw.hypervisor_hart.hypervisor_hart_state.csrs.mcause.read();
    ctx.csr.medeleg = hw.hypervisor_hart.hypervisor_hart_state.csrs.medeleg.read();
    ctx.csr.mideleg = hw.hypervisor_hart.hypervisor_hart_state.csrs.mideleg.read();
    ctx.csr.mie = hw.hypervisor_hart.hypervisor_hart_state.csrs.mie.read();
    ctx.csr.mip = hw.hypervisor_hart.hypervisor_hart_state.csrs.mip.read();
    ctx.csr.mstatus = hw.hypervisor_hart.hypervisor_hart_state.csrs.mstatus.read();
    ctx.csr.mtinst = hw.hypervisor_hart.hypervisor_hart_state.csrs.mtinst.read();
    ctx.csr.mtval = hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval.read();
    ctx.csr.mtval2 = hw.hypervisor_hart.hypervisor_hart_state.csrs.mtval2.read();
    ctx.csr.mtvec = hw.hypervisor_hart.hypervisor_hart_state.csrs.mtvec.read();
    ctx.csr.mscratch = hw.hypervisor_hart.hypervisor_hart_state.csrs.mscratch.read();

    // S mode registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sstatus = ReadWriteRiscvCsr(ctx.csr.sstatus);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sie = ReadWriteRiscvCsr(ctx.csr.sie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stvec = ReadWriteRiscvCsr(ctx.csr.stvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scounteren = ReadWriteRiscvCsr(ctx.csr.scounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.senvcfg = ReadWriteRiscvCsr(ctx.csr.senvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sscratch = ReadWriteRiscvCsr(ctx.csr.sscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.sepc = ReadWriteRiscvCsr(ctx.csr.sepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.scause = ReadWriteRiscvCsr(ctx.csr.scause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.stval = ReadWriteRiscvCsr(ctx.csr.stval);
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.sip = ReadWriteRiscvCsr(ctx.csr.sip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.satp = ReadWriteRiscvCsr(ctx.csr.satp);

    // HS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hstatus = ReadWriteRiscvCsr(ctx.csr.hstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hedeleg = ReadWriteRiscvCsr(ctx.csr.hedeleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hideleg = ReadWriteRiscvCsr(ctx.csr.hideleg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hie = ReadWriteRiscvCsr(ctx.csr.hie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hcounteren = ReadWriteRiscvCsr(ctx.csr.hcounteren);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeie = ReadWriteRiscvCsr(ctx.csr.hgeie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htval = ReadWriteRiscvCsr(ctx.csr.htval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hip = ReadWriteRiscvCsr(ctx.csr.hip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hvip = ReadWriteRiscvCsr(ctx.csr.hvip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htinst = ReadWriteRiscvCsr(ctx.csr.htinst);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeip = ReadWriteRiscvCsr(ctx.csr.hgeip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.henvcfg = ReadWriteRiscvCsr(ctx.csr.henvcfg);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.hgatp = ReadWriteRiscvCsr(ctx.csr.hgatp);

    // HS mode debug registers
    // hw.hypervisor_hart.hypervisor_hart_state.csrs.hcontext = ReadWriteRiscvCsr(ctx.csr.hcontext);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.htimedelta = ReadWriteRiscvCsr(ctx.csr.htimedelta);

    // VS mode registers
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsstatus = ReadWriteRiscvCsr(ctx.csr.vsstatus);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsie = ReadWriteRiscvCsr(ctx.csr.vsie);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsip = ReadWriteRiscvCsr(ctx.csr.vsip);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstvec = ReadWriteRiscvCsr(ctx.csr.vstvec);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsscratch = ReadWriteRiscvCsr(ctx.csr.vsscratch);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsepc = ReadWriteRiscvCsr(ctx.csr.vsepc);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vscause = ReadWriteRiscvCsr(ctx.csr.vscause);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vstval = ReadWriteRiscvCsr(ctx.csr.vstval);
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsatp = ReadWriteRiscvCsr(ctx.csr.vsatp);
}