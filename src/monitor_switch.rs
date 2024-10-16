use crate::ace::core::architecture::control_status_registers::ReadWriteRiscvCsr;
use crate::ace::core::control_data::HardwareHart;
use crate::host::MiralisContext;
use crate::policy::Policy;
use crate::virt::VirtContext;

pub fn overwrite_hardware_hart_with_virtctx(hw: &mut HardwareHart, ctx: &mut VirtContext) {
    // Save normal registers
    for i in 0..32 {
        hw.hypervisor_hart.hypervisor_hart_state.gprs.0[i] = ctx.regs[i]
    }

    // And restore CSR registers from main memory
    hw.hypervisor_hart.hypervisor_hart_state.csrs.save_in_main_memory();
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
    // ctx.csr.sstatus = hw.hypervisor_hart.hypervisor_hart_state.csrs.sstatus.read();
    // ctx.csr.sie = hw.hypervisor_hart.hypervisor_hart_state.csrs.sie.read();
    ctx.csr.stvec = hw.hypervisor_hart.hypervisor_hart_state.csrs.stvec.read();
    ctx.csr.scounteren = hw.hypervisor_hart.hypervisor_hart_state.csrs.scounteren.read();
    ctx.csr.senvcfg = hw.hypervisor_hart.hypervisor_hart_state.csrs.senvcfg.read();
    ctx.csr.sscratch = hw.hypervisor_hart.hypervisor_hart_state.csrs.sscratch.read();
    ctx.csr.sepc = hw.hypervisor_hart.hypervisor_hart_state.csrs.sepc.read();
    ctx.csr.scause = hw.hypervisor_hart.hypervisor_hart_state.csrs.scause.read();
    ctx.csr.stval = hw.hypervisor_hart.hypervisor_hart_state.csrs.stval.read();
    // ctx.csr.sip= hw.hypervisor_hart.hypervisor_hart_state.csrs.sip.read();
    ctx.csr.satp = hw.hypervisor_hart.hypervisor_hart_state.csrs.satp.read();

    // HS mode registers
    /*ctx.csr.hstatus = hw.hypervisor_hart.hypervisor_hart_state.csrs.hstatus.read();
    ctx.csr.hedeleg = hw.hypervisor_hart.hypervisor_hart_state.csrs.hedeleg.read();
    ctx.csr.hideleg = hw.hypervisor_hart.hypervisor_hart_state.csrs.hideleg.read();
    ctx.csr.hie = hw.hypervisor_hart.hypervisor_hart_state.csrs.hie.read();
    ctx.csr.hcounteren = hw.hypervisor_hart.hypervisor_hart_state.csrs.hcounteren.read();
    ctx.csr.hgeie = hw.hypervisor_hart.hypervisor_hart_state.csrs.hgeie.read();
    ctx.csr.htval = hw.hypervisor_hart.hypervisor_hart_state.csrs.htval.read();
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
    hw.hypervisor_hart.hypervisor_hart_state.csrs.vsatp = ReadWriteRiscvCsr(ctx.csr.vsatp);*/
}

pub fn address_to_virt_context<'a>(addr: usize) -> &'a mut VirtContext {
    // Convert usize to a raw pointer
    let ptr = addr as *mut VirtContext;

    // Unsafe block required to dereference the raw pointer
    unsafe {
        &mut *ptr // Dereference the pointer to get a mutable reference
    }
}

pub fn address_to_miralis_context<'a>(addr: usize) -> &'a mut MiralisContext {
    // Convert usize to a raw pointer
    let ptr = addr as *mut MiralisContext;

    // Unsafe block required to dereference the raw pointer
    unsafe {
        &mut *ptr // Dereference the pointer to get a mutable reference
    }
}

pub fn address_to_policy<'a>(addr: usize) -> &'a mut Policy {
    // Convert usize to a raw pointer
    let ptr = addr as *mut Policy;

    // Unsafe block required to dereference the raw pointer
    unsafe {
        &mut *ptr // Dereference the pointer to get a mutable reference
    }
}
