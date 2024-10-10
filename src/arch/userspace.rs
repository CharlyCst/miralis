//! A mock of architecture specific features when running in user space.
//!
//! This implementation is useful for running Miralis on the host (potentially non-riscv)
//! architecutre, such as when running unit tests.

use core::marker::PhantomData;
use core::ptr;

use spin::Mutex;

use super::{
    mie, mstatus, parse_mpp_return_mode, Architecture, Csr, ExtensionsCapability, MCause, Mode,
};
use crate::arch::pmp::PmpFlush;
use crate::arch::{HardwareCapability, PmpGroup};
use crate::decoder::Instr;
use crate::main;
use crate::virt::VirtContext;

static HOST_CTX: Mutex<VirtContext> = Mutex::new(VirtContext::new(
    0,
    16,
    ExtensionsCapability {
        has_h_extension: false,
        has_s_extension: true,
        _has_f_extension: false,
        _has_d_extension: false,
        _has_q_extension: false,
    },
));

/// User space mock, running on the host architecture.
pub struct HostArch {}

impl Architecture for HostArch {
    fn init() {
        // Use main to avoid "never used" warnings.
        let _ = main;

        todo!()
    }

    fn wfi() {
        log::debug!("Userspace wfi");
    }

    unsafe fn set_mpp(mode: Mode) -> Mode {
        let value = mode.to_bits() << mstatus::MPP_OFFSET;
        let prev_mstatus = Self::read_csr(Csr::Mstatus);
        Self::write_csr(Csr::Mstatus, (prev_mstatus & !mstatus::MPP_FILTER) | value);
        parse_mpp_return_mode(prev_mstatus)
    }

    unsafe fn write_pmp(pmp: &PmpGroup) -> PmpFlush {
        let pmpaddr = pmp.pmpaddr();
        let pmpcfg = pmp.pmpcfg();
        let nb_pmp = pmp.nb_pmp as usize;

        assert!(
            nb_pmp <= pmpaddr.len() && nb_pmp <= pmpcfg.len() * 8,
            "Invalid number of PMP registers"
        );

        HOST_CTX.lock().csr.pmpaddr[..nb_pmp].copy_from_slice(&pmpaddr[..nb_pmp]);

        for (idx, _) in pmpcfg.iter().enumerate().take(nb_pmp / 8) {
            let cfg = pmpcfg[idx];
            HOST_CTX.lock().csr.pmpcfg[idx * 2] = cfg;
        }

        PmpFlush()
    }

    unsafe fn run_vcpu(_ctx: &mut crate::virt::VirtContext) {
        todo!()
    }

    unsafe fn get_raw_faulting_instr(trap_info: &super::TrapInfo) -> usize {
        if trap_info.mcause == MCause::IllegalInstr as usize {
            // First, try mtval and check if it contains an instruction
            if trap_info.mtval != 0 {
                return trap_info.mtval;
            }
        }

        let instr_ptr = trap_info.mepc as *const u32;

        // With compressed instruction extention ("C") instructions can be misaligned.
        // TODO: add support for 16 bits instructions
        let instr = ptr::read_unaligned(instr_ptr);
        instr as usize
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
            interrupts: usize::MAX,
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
                _has_f_extension: false,
                _has_d_extension: false,
                _has_q_extension: false,
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
            Csr::Mvendorid => ctx.csr.mvendorid,
            Csr::Marchid => ctx.csr.marchid,
            Csr::Mimpid => ctx.csr.mimpid,
            Csr::Pmpcfg(index) => ctx.csr.pmpcfg[index],
            Csr::Pmpaddr(index) => ctx.csr.pmpaddr[index],
            Csr::Mcycle => ctx.csr.mcycle,
            Csr::Minstret => ctx.csr.minstret,
            Csr::Mhpmcounter(index) => ctx.csr.mhpmcounter[index],
            Csr::Mcountinhibit => ctx.csr.mcountinhibit,
            Csr::Mhpmevent(index) => ctx.csr.mhpmevent[index],
            Csr::Mcounteren => ctx.csr.mcounteren,
            Csr::Menvcfg => ctx.csr.menvcfg,
            Csr::Mseccfg => ctx.csr.mseccfg,
            Csr::Mconfigptr => ctx.csr.mconfigptr,
            Csr::Medeleg => ctx.csr.medeleg,
            Csr::Mideleg => ctx.csr.mideleg,
            Csr::Mtinst => ctx.csr.mtinst,
            Csr::Mtval2 => todo!(),
            Csr::Tselect => todo!(),
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
            Csr::Scounteren => ctx.csr.scounteren,
            Csr::Senvcfg => ctx.csr.senvcfg,
            Csr::Sscratch => ctx.csr.sscratch,
            Csr::Sepc => ctx.csr.sepc,
            Csr::Scause => ctx.csr.scause,
            Csr::Stval => ctx.csr.stval,
            Csr::Sip => ctx.csr.mip & mie::SIE_FILTER,
            Csr::Satp => ctx.csr.satp,
            Csr::Scontext => ctx.csr.scontext,
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
            Csr::Mvendorid => ctx.csr.mvendorid = value,
            Csr::Marchid => ctx.csr.marchid = value,
            Csr::Mimpid => ctx.csr.mimpid = value,
            Csr::Pmpcfg(index) => ctx.csr.pmpcfg[index] = value,
            Csr::Pmpaddr(index) => ctx.csr.pmpaddr[index] = value,
            Csr::Mcycle => ctx.csr.mcycle = value,
            Csr::Minstret => ctx.csr.minstret = value,
            Csr::Mhpmcounter(index) => ctx.csr.mhpmcounter[index] = value,
            Csr::Mcountinhibit => ctx.csr.mcountinhibit = value,
            Csr::Mhpmevent(index) => ctx.csr.mhpmevent[index] = value,
            Csr::Mcounteren => ctx.csr.mcounteren = value,
            Csr::Menvcfg => ctx.csr.menvcfg = value,
            Csr::Mseccfg => ctx.csr.mseccfg = value,
            Csr::Mconfigptr => ctx.csr.mconfigptr = value,
            Csr::Medeleg => ctx.csr.medeleg = value,
            Csr::Mideleg => ctx.csr.mideleg = value,
            Csr::Mtinst => ctx.csr.mtinst = value,
            Csr::Mtval2 => todo!(),
            Csr::Tselect => todo!(),
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
            Csr::Scounteren => ctx.csr.scounteren = value,
            Csr::Senvcfg => ctx.csr.senvcfg = value,
            Csr::Sscratch => ctx.csr.sscratch = value,
            Csr::Sepc => ctx.csr.sepc = value,
            Csr::Scause => ctx.csr.scause = value,
            Csr::Stval => ctx.csr.stval = value,
            Csr::Sip => ctx.csr.mip = ctx.csr.mip & !mie::SIE_FILTER | value & mie::SIE_FILTER,
            Csr::Satp => ctx.csr.satp = value,
            Csr::Scontext => ctx.csr.scontext = value,
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
            Csr::Unknown => panic!("Unkown csr!"),
        }
        prev_val
    }

    unsafe fn clear_csr_bits(csr: Csr, bits_mask: usize) {
        Self::write_csr(csr, Self::read_csr(csr) & !bits_mask);
    }

    unsafe fn set_csr_bits(csr: Csr, bits_mask: usize) {
        Self::write_csr(csr, Self::read_csr(csr) | bits_mask);
    }

    unsafe fn handle_virtual_load_store(_instr: Instr, _ctx: &mut VirtContext) {
        todo!();
    }

    unsafe fn read_bytes_from_mode(
        _src: *const u8,
        _dest: &mut [u8],
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!();
    }

    fn install_handler(_: usize) {
        todo!();
    }

    unsafe fn store_bytes_from_mode(
        _src: &mut [u8],
        _dest: *const u8,
        _mode: Mode,
    ) -> Result<(), ()> {
        todo!()
    }
}
