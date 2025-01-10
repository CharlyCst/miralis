//! A mock of architecture specific features when running in user space.
//!
//! This implementation is useful for running Miralis on the host (potentially non-riscv)
//! architecture, such as when running unit tests.

use core::marker::PhantomData;

use spin::{Mutex, MutexGuard};

use super::{mie, mstatus, Architecture, Csr, ExtensionsCapability, Mode};
use crate::arch::HardwareCapability;
use crate::decoder::{LoadInstr, StoreInstr};
use crate::virt::VirtContext;

pub static HOST_CTX: Mutex<VirtContext> = Mutex::new(VirtContext::new(
    0,
    16,
    ExtensionsCapability {
        has_h_extension: false,
        has_s_extension: true,
        has_sstc_extension: false,
        is_sstc_enabled: false,
        has_v_extension: false,
        has_crypto_extension: false,
        has_zicntr: true,
        has_zihpm_extension: true,
    },
));

pub fn return_userspace_ctx() -> MutexGuard<'static, VirtContext> {
    HOST_CTX.lock()
}

/// User space mock, running on the host architecture.
pub struct HostArch {}

impl Architecture for HostArch {
    fn init() {
        todo!()
    }

    fn wfi() {
        log::debug!("Userspace wfi");
    }

    unsafe fn write_pmpaddr(idx: usize, value: usize) {
        HOST_CTX.lock().csr.pmpaddr[idx] = value;
    }

    unsafe fn write_pmpcfg(idx: usize, configuration: usize) {
        // The pmpcfg registers must be even. Since we have an array of 8 elements, we divide all indices by two
        HOST_CTX.lock().csr.pmpcfg[idx / 2] = configuration;
    }

    unsafe fn run_vcpu(_ctx: &mut crate::virt::VirtContext) {
        todo!()
    }

    unsafe fn sfencevma(_vaddr: Option<usize>, _asid: Option<usize>) {
        log::debug!("Userspace sfencevma");
    }

    unsafe fn hfencegvma(_: Option<usize>, _: Option<usize>) {
        log::debug!("Userspace hfencegvma")
    }

    unsafe fn hfencevvma(_: Option<usize>, _: Option<usize>) {
        log::debug!("Userspace hfencevvma")
    }

    unsafe fn detect_hardware() -> HardwareCapability {
        HardwareCapability {
            interrupts: mie::ALL_INT,
            hart: 0,
            _marker: PhantomData,
            available_reg: super::RegistersCapability {
                menvcfg: true,
                senvcfg: true,
                nb_pmp: 16,
            },
            extensions: ExtensionsCapability {
                has_h_extension: false,
                has_s_extension: true,
                has_v_extension: true,
                has_crypto_extension: false,
                has_zicntr: true,
                has_sstc_extension: false,
                is_sstc_enabled: false,
                has_zihpm_extension: true,
            },
        }
    }

    fn read_csr(csr: Csr) -> usize {
        let ctx = HOST_CTX.lock();
        match csr {
            Csr::Mhartid => ctx.csr.marchid,
            Csr::Mstatus => ctx.csr.mstatus,
            Csr::Misa => ctx.csr.misa,
            Csr::Mie => ctx.csr.mie,
            Csr::Mtvec => ctx.csr.mtvec,
            Csr::Mscratch => ctx.csr.mscratch,
            Csr::Mip => ctx.csr.mip,
            Csr::Mvendorid => ctx.csr.mvendorid as usize,
            Csr::Marchid => ctx.csr.marchid,
            Csr::Mimpid => ctx.csr.mimpid,
            Csr::Pmpcfg(index) => ctx.csr.pmpcfg[index],
            Csr::Pmpaddr(index) => ctx.csr.pmpaddr[index],
            Csr::Mcycle => ctx.csr.mcycle,
            Csr::Minstret => ctx.csr.minstret,
            Csr::Mhpmcounter(index) => ctx.csr.mhpmcounter[index],
            Csr::Mcountinhibit => ctx.csr.mcountinhibit as usize,
            Csr::Mhpmevent(index) => ctx.csr.mhpmevent[index],
            Csr::Mcounteren => ctx.csr.mcounteren as usize,
            Csr::Menvcfg => ctx.csr.menvcfg,
            Csr::Mseccfg => ctx.csr.mseccfg,
            Csr::Mconfigptr => ctx.csr.mconfigptr,
            Csr::Medeleg => ctx.csr.medeleg,
            Csr::Mideleg => ctx.csr.mideleg,
            Csr::Mtinst => ctx.csr.mtinst,
            Csr::Mtval2 => todo!(),
            Csr::Tselect => ctx.csr.tselect,
            Csr::Tdata1 => todo!(),
            Csr::Tdata2 => todo!(),
            Csr::Tdata3 => todo!(),
            Csr::Mcontext => todo!(),
            Csr::Dcsr => todo!(),
            Csr::Dpc => todo!(),
            Csr::Dscratch0 => todo!(),
            Csr::Dscratch1 => todo!(),
            Csr::Mepc => ctx.csr.mepc,
            Csr::Mcause => ctx.csr.mcause,
            Csr::Mtval => ctx.csr.mtval,
            Csr::Sstatus => ctx.csr.mstatus & mstatus::SSTATUS_FILTER,
            Csr::Sie => ctx.csr.mie & mie::SIE_FILTER,
            Csr::Stvec => ctx.csr.stvec,
            Csr::Scounteren => ctx.csr.scounteren as usize,
            Csr::Senvcfg => ctx.csr.senvcfg,
            Csr::Sscratch => ctx.csr.sscratch,
            Csr::Sepc => ctx.csr.sepc,
            Csr::Scause => ctx.csr.scause,
            Csr::Stval => ctx.csr.stval,
            Csr::Sip => ctx.csr.mip & mie::SIE_FILTER,
            Csr::Satp => ctx.csr.satp,
            Csr::Scontext => ctx.csr.scontext,
            Csr::Stimecmp => ctx.csr.stimecmp,
            Csr::Hstatus => ctx.csr.hstatus,
            Csr::Hedeleg => ctx.csr.hedeleg,
            Csr::Hideleg => ctx.csr.hideleg,
            Csr::Hvip => ctx.csr.hvip,
            Csr::Hip => ctx.csr.hip,
            Csr::Hie => ctx.csr.hie,
            Csr::Hgeip => ctx.csr.hgeip,
            Csr::Hgeie => ctx.csr.hgeie,
            Csr::Henvcfg => ctx.csr.henvcfg,
            Csr::Hcounteren => ctx.csr.hcounteren,
            Csr::Htimedelta => ctx.csr.htimedelta,
            Csr::Htval => ctx.csr.htval,
            Csr::Htinst => ctx.csr.htinst,
            Csr::Hgatp => ctx.csr.hgatp,
            Csr::Vsstatus => ctx.csr.vsstatus,
            Csr::Vsie => ctx.csr.vsie,
            Csr::Vstvec => ctx.csr.vstvec,
            Csr::Vsscratch => ctx.csr.vsscratch,
            Csr::Vsepc => ctx.csr.vsepc,
            Csr::Vscause => ctx.csr.vscause,
            Csr::Vstval => ctx.csr.vstval,
            Csr::Vsip => ctx.csr.vsip,
            Csr::Vsatp => ctx.csr.vsatp,
            Csr::Vstart => ctx.csr.vstart as usize,
            Csr::Vxsat => {
                if ctx.csr.vxsat {
                    1
                } else {
                    0
                }
            }
            Csr::Vxrm => ctx.csr.vxrm as usize,
            Csr::Vcsr => ctx.csr.vcsr as usize,
            Csr::Vl => ctx.csr.vl,
            Csr::Vtype => ctx.csr.vtype,
            Csr::Vlenb => ctx.csr.vlenb,
            Csr::Cycle => ctx.csr.mcycle,
            Csr::Time => 0,
            Csr::Instret => ctx.csr.minstret,
            Csr::Seed => 0x80000000, // Magic value, used for model checking
            Csr::Unknown => panic!("Unkown csr!"),
        }
    }

    unsafe fn write_csr(csr: Csr, value: usize) -> usize {
        let prev_val = Self::read_csr(csr);
        let mut ctx = HOST_CTX.lock();
        match csr {
            Csr::Mhartid => ctx.csr.marchid = value,
            Csr::Mstatus => ctx.csr.mstatus = value,
            Csr::Misa => ctx.csr.misa = value,
            Csr::Mie => ctx.csr.mie = value,
            Csr::Mtvec => ctx.csr.mtvec = value,
            Csr::Mscratch => ctx.csr.mscratch = value,
            Csr::Mip => ctx.csr.mip = value, // TODO : add write filter
            Csr::Mvendorid => ctx.csr.mvendorid = value as u32,
            Csr::Marchid => ctx.csr.marchid = value,
            Csr::Mimpid => ctx.csr.mimpid = value,
            Csr::Pmpcfg(index) => ctx.csr.pmpcfg[index] = value,
            Csr::Pmpaddr(index) => ctx.csr.pmpaddr[index] = value,
            Csr::Mcycle => ctx.csr.mcycle = value,
            Csr::Minstret => ctx.csr.minstret = value,
            Csr::Mhpmcounter(index) => ctx.csr.mhpmcounter[index] = value,
            Csr::Mcountinhibit => ctx.csr.mcountinhibit = value as u32,
            Csr::Mhpmevent(index) => ctx.csr.mhpmevent[index] = value,
            Csr::Mcounteren => ctx.csr.mcounteren = value as u32,
            Csr::Menvcfg => ctx.csr.menvcfg = value,
            Csr::Mseccfg => ctx.csr.mseccfg = value,
            Csr::Mconfigptr => ctx.csr.mconfigptr = value,
            Csr::Medeleg => ctx.csr.medeleg = value,
            Csr::Mideleg => ctx.csr.mideleg = value,
            Csr::Mtinst => ctx.csr.mtinst = value,
            Csr::Mtval2 => todo!(),
            Csr::Tselect => ctx.csr.tselect = value,
            Csr::Tdata1 => todo!(),
            Csr::Tdata2 => todo!(),
            Csr::Tdata3 => todo!(),
            Csr::Mcontext => todo!(),
            Csr::Dcsr => todo!(),
            Csr::Dpc => todo!(),
            Csr::Dscratch0 => todo!(),
            Csr::Dscratch1 => todo!(),
            Csr::Mepc => ctx.csr.mepc = value,
            Csr::Mcause => ctx.csr.mcause = value,
            Csr::Mtval => ctx.csr.mtval = value,
            Csr::Sstatus => {
                ctx.csr.mstatus =
                    (ctx.csr.mstatus & !mstatus::SSTATUS_FILTER) | (value & mstatus::SSTATUS_FILTER)
            }
            Csr::Sie => ctx.csr.mie = (ctx.csr.mie & !mie::SIE_FILTER) & (value & mie::SIE_FILTER),
            Csr::Stvec => ctx.csr.stvec = value,
            Csr::Scounteren => ctx.csr.scounteren = value as u32,
            Csr::Senvcfg => ctx.csr.senvcfg = value,
            Csr::Sscratch => ctx.csr.sscratch = value,
            Csr::Sepc => ctx.csr.sepc = value,
            Csr::Scause => ctx.csr.scause = value,
            Csr::Stval => ctx.csr.stval = value,
            Csr::Sip => ctx.csr.mip = ctx.csr.mip & !mie::SIE_FILTER | value & mie::SIE_FILTER,
            Csr::Satp => ctx.csr.satp = value,
            Csr::Scontext => ctx.csr.scontext = value,
            Csr::Stimecmp => ctx.csr.stimecmp = value,
            Csr::Hstatus => ctx.csr.hstatus = value,
            Csr::Hedeleg => ctx.csr.hedeleg = value,
            Csr::Hideleg => ctx.csr.hideleg = value,
            Csr::Hvip => ctx.csr.hvip = value,
            Csr::Hip => ctx.csr.hip = value,
            Csr::Hie => ctx.csr.hie = value,
            Csr::Hgeip => {}
            Csr::Hgeie => ctx.csr.hgeie = value,
            Csr::Henvcfg => ctx.csr.henvcfg = value,
            Csr::Hcounteren => ctx.csr.hcounteren = value,
            Csr::Htimedelta => ctx.csr.htimedelta = value,
            Csr::Htval => ctx.csr.htval = value,
            Csr::Htinst => ctx.csr.htinst = value,
            Csr::Hgatp => ctx.csr.hgatp = value,
            Csr::Vsstatus => ctx.csr.vsstatus = value,
            Csr::Vsie => ctx.csr.vsie = value,
            Csr::Vstvec => ctx.csr.vstvec = value,
            Csr::Vsscratch => ctx.csr.vsscratch = value,
            Csr::Vsepc => ctx.csr.vsepc = value,
            Csr::Vscause => ctx.csr.vscause = value,
            Csr::Vstval => ctx.csr.vstval = value,
            Csr::Vsip => ctx.csr.vsip = value,
            Csr::Vsatp => ctx.csr.vsatp = value,
            Csr::Vstart => ctx.csr.vstart = value as u16,
            Csr::Vxsat => ctx.csr.vxsat = (value & 0x1) != 0,
            Csr::Vxrm => ctx.csr.vxrm = (value & 0b11) as u8,
            Csr::Vcsr => ctx.csr.vcsr = (value & 0b111) as u8,
            Csr::Vl => ctx.csr.vl = value,
            Csr::Vtype => ctx.csr.vtype = value,
            Csr::Vlenb => ctx.csr.vlenb = value,
            Csr::Cycle => {}
            Csr::Time => {}
            Csr::Instret => {}
            Csr::Seed => (), // Read only
            Csr::Unknown => panic!("Unkown csr!"),
        }
        prev_val
    }

    unsafe fn clear_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        Self::write_csr(csr, Self::read_csr(csr) & !bits_mask)
    }

    unsafe fn set_csr_bits(csr: Csr, bits_mask: usize) -> usize {
        Self::write_csr(csr, Self::read_csr(csr) | bits_mask)
    }

    unsafe fn read_bytes_from_mode(
        _src: *const u8,
        _dest: &mut [u8],
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!();
    }

    unsafe fn store_bytes_from_mode(
        _src: &mut [u8],
        _dest: *const u8,
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!()
    }

    unsafe fn handle_virtual_load(_instr: LoadInstr, _ctx: &mut VirtContext) {
        todo!()
    }

    unsafe fn handle_virtual_store(_instr: StoreInstr, _ctx: &mut VirtContext) {
        todo!()
    }
}
