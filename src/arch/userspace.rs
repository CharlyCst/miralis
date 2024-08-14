//! A mock of architecture specific features when running in user space.
//!
//! This implementation is useful for running Miralis on the host (potentially non-riscv)
//! architecutre, such as when running unit tests.

use core::marker::PhantomData;
use core::{ptr, usize};

use spin::Mutex;

use super::{mie, mstatus, Architecture, Csr, MCause, Mode};
use crate::arch::{HardwareCapability, PmpGroup};
use crate::main;
use crate::virt::VirtContext;

static HOST_CTX: Mutex<VirtContext> = Mutex::new(VirtContext::new(0, 16));

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

    unsafe fn set_mpp(mode: Mode) {
        let value = mode.to_bits() << mstatus::MPP_OFFSET;
        let mstatus = Self::read_csr(Csr::Mstatus);
        Self::write_csr(Csr::Mstatus, (mstatus & !mstatus::MPP_FILTER) | value);
    }

    unsafe fn write_pmp(pmp: &PmpGroup) {
        let pmpaddr = pmp.pmpaddr();
        let pmpcfg = pmp.pmpcfg();
        let nb_pmp = pmp.nb_pmp as usize;

        assert!(
            nb_pmp as usize <= pmpaddr.len() && nb_pmp as usize <= pmpcfg.len() * 8,
            "Invalid number of PMP registers"
        );

        for idx in 0..nb_pmp {
            HOST_CTX.lock().csr.pmpaddr[idx] = pmpaddr[idx];
        }
        for idx in 0..(nb_pmp / 8) {
            let cfg = pmpcfg[idx];
            HOST_CTX.lock().csr.pmpcfg[idx * 2] = cfg;
        }
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

    unsafe fn sfence_vma() {
        log::debug!("Userspace sfence.vma");
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
}
