//! Firmware Virtualisation

mod csr;
mod emulator;
pub mod memory;
mod world_switch;

pub use csr::traits;
pub use emulator::ExitResult;

use crate::arch::{mie, misa, ExtensionsCapability, Mode, TrapInfo};

/// The execution mode, either virtualized firmware or native payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Virtualized virmware, running in U-mode.
    Firmware,
    /// Native payload, running in U or S-mode.
    Payload,
}

/// The context of a virtual firmware.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    pub regs: [usize; 32],
    /// Program Counter
    pub pc: usize,
    /// Information on the trap that ocurred, used to handle traps
    pub trap_info: TrapInfo,
    /// Virtual Control and Status Registers
    pub csr: VirtCsr,
    /// Current privilege mode
    pub mode: Mode,
    /// Number of virtual PMPs
    pub nb_pmp: usize,
    /// The PMP granularity (G)
    ///
    /// The minimal sizes of regions protected by the PMP is 2^(G+2). See specification for
    /// detailled explanation of the PMP grain.
    pub pmp_grain: usize,
    /// Availables RISC-V extensions
    pub extensions: ExtensionsCapability,
    /// Hart ID
    pub hart_id: usize,
    /// Number of exists to Miralis
    pub nb_exits: usize,
    /// Wether the vCPU is currently in Wait For Interrupt mode (WFI)
    pub is_wfi: bool,
}

impl VirtContext {
    pub const fn new(
        hart_id: usize,
        nb_pmp_registers_left: usize,
        available_extension: ExtensionsCapability,
    ) -> Self {
        assert!(nb_pmp_registers_left <= 64, "Too many PMP registers");

        VirtContext {
            host_stack: 0,
            regs: [0; 32],
            csr: VirtCsr {
                misa: 0,
                mie: 0,
                mip: 0,
                mtvec: 0,
                mscratch: 0,
                mvendorid: 0,
                marchid: 0,
                mimpid: 0,
                mcycle: 0,
                minstret: 0,
                mcountinhibit: 0,
                mcounteren: 0,
                menvcfg: 0,
                mseccfg: 0,
                mcause: 0,
                tselect: 0,
                mepc: 0,
                mtval: 0,
                mtval2: 0,
                mstatus: 0,
                mtinst: 0,
                mconfigptr: 0,
                stvec: 0,
                scounteren: 0,
                senvcfg: 0,
                sscratch: 0,
                sepc: 0,
                scause: 0,
                stval: 0,
                satp: 0,
                scontext: 0,
                stimecmp: 0,
                medeleg: 0,
                mideleg: mie::MIDELEG_READ_ONLY_ONE,
                hstatus: 0,
                hedeleg: 0,
                hideleg: 0,
                hvip: 0,
                hip: 0,
                hie: 0,
                hgeip: 0,
                hgeie: 0,
                henvcfg: 0,
                henvcfgh: 0,
                hcounteren: 0,
                htimedelta: 0,
                htimedeltah: 0,
                htval: 0,
                htinst: 0,
                hgatp: 0,
                vsstatus: 0,
                vsie: 0,
                vstvec: 0,
                vsscratch: 0,
                vsepc: 0,
                vscause: 0,
                vstval: 0,
                vsip: 0,
                vsatp: 0,
                pmpcfg: [0; 8],
                pmpaddr: [0; 64],
                mhpmcounter: [0; 29],
                mhpmevent: [0; 29],
                vstart: 0,
                vxsat: false,
                vxrm: 0,
                vcsr: 0,
                vl: 0,
                vtype: 0,
                vlenb: 0,
            },
            pc: 0,
            mode: Mode::M,
            nb_pmp: nb_pmp_registers_left,
            pmp_grain: 0,
            trap_info: TrapInfo {
                mepc: 0,
                mstatus: 0,
                mcause: 0,
                mip: 0,
                mtval: 0,
                mtval2: 0,
                mtinst: 0,
                gva: false,
            },
            nb_exits: 0,
            hart_id,
            extensions: available_extension,
            is_wfi: false,
        }
    }

    /// Expected PC alignment, depending on the C extension.
    pub fn pc_alignment_mask(&self) -> usize {
        if self.csr.misa & misa::C != 0 {
            !0b00
        } else {
            !0b10
        }
    }
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct VirtCsr {
    pub misa: usize,
    pub mie: usize,
    pub mip: usize,
    pub mtvec: usize,
    pub mvendorid: u32,
    pub marchid: usize,
    pub mimpid: usize,
    pub mcycle: usize,
    pub minstret: usize,
    pub mscratch: usize,
    pub mcountinhibit: u32,
    pub mcounteren: u32,
    pub menvcfg: usize,
    pub mseccfg: usize,
    pub mcause: usize,
    pub tselect: usize,
    pub mepc: usize,
    pub mtval: usize,
    pub mtval2: usize,
    pub mstatus: usize,
    pub mtinst: usize,
    pub mconfigptr: usize,
    pub stvec: usize,
    pub scounteren: u32,
    pub senvcfg: usize,
    pub sscratch: usize,
    pub sepc: usize,
    pub scause: usize,
    pub stval: usize,
    pub satp: usize,
    pub scontext: usize,
    pub stimecmp: usize,
    pub medeleg: usize,
    pub mideleg: usize,
    pub hstatus: usize,
    pub hedeleg: usize,
    pub hideleg: usize,
    pub hvip: usize,
    pub hip: usize,
    pub hie: usize,
    pub hgeip: usize,
    pub hgeie: usize,
    pub henvcfg: usize,
    pub henvcfgh: usize,
    pub hcounteren: usize,
    pub htimedelta: usize,
    pub htimedeltah: usize,
    pub htval: usize,
    pub htinst: usize,
    pub hgatp: usize,
    pub vsstatus: usize,
    pub vsie: usize,
    pub vstvec: usize,
    pub vsscratch: usize,
    pub vsepc: usize,
    pub vscause: usize,
    pub vstval: usize,
    pub vsip: usize,
    pub vsatp: usize,
    pub pmpcfg: [usize; 8],
    pub pmpaddr: [usize; 64],
    pub mhpmcounter: [usize; 29],
    pub mhpmevent: [usize; 29],
    pub vstart: u16,
    pub vxsat: bool,
    pub vxrm: u8, // 2 bits wide
    pub vcsr: u8, // 3 bits wide
    pub vl: usize,
    pub vtype: usize,
    pub vlenb: usize,
}

impl VirtCsr {
    pub fn new_empty() -> Self {
        VirtCsr {
            misa: 0,
            mie: 0,
            mip: 0,
            mtvec: 0,
            mvendorid: 0,
            marchid: 0,
            mimpid: 0,
            mcycle: 0,
            minstret: 0,
            mscratch: 0,
            mcountinhibit: 0,
            mcounteren: 0,
            menvcfg: 0,
            mseccfg: 0,
            mcause: 0,
            tselect: 0,
            mepc: 0,
            mtval: 0,
            mtval2: 0,
            mstatus: 0,
            mtinst: 0,
            mconfigptr: 0,
            stvec: 0,
            scounteren: 0,
            senvcfg: 0,
            sscratch: 0,
            sepc: 0,
            scause: 0,
            stval: 0,
            satp: 0,
            scontext: 0,
            stimecmp: 0,
            medeleg: 0,
            mideleg: 0,
            hstatus: 0,
            hedeleg: 0,
            hideleg: 0,
            hvip: 0,
            hip: 0,
            hie: 0,
            hgeip: 0,
            hgeie: 0,
            henvcfg: 0,
            henvcfgh: 0,
            hcounteren: 0,
            htimedelta: 0,
            htimedeltah: 0,
            htval: 0,
            htinst: 0,
            hgatp: 0,
            vsstatus: 0,
            vsie: 0,
            vstvec: 0,
            vsscratch: 0,
            vsepc: 0,
            vscause: 0,
            vstval: 0,
            vsip: 0,
            vsatp: 0,
            pmpcfg: [0; 8],
            pmpaddr: [0; 64],
            mhpmcounter: [0; 29],
            mhpmevent: [0; 29],
            vstart: 0,
            vxsat: false,
            vxrm: 0,
            vcsr: 0,
            vl: 0,
            vtype: 0,
            vlenb: 0,
        }
    }

    pub fn set_csr_field(csr: &mut usize, offset: usize, filter: usize, value: usize) {
        debug_assert_eq!((value << offset) & !filter, 0, "Invalid set_csr_field");
        // Clear field
        *csr &= !filter;
        // Set field
        *csr |= value << offset;
    }

    /// Returns the mask of valid bit for the given PMP configuration register.
    pub fn get_pmp_cfg_filter(pmp_csr_idx: usize, nbr_valid_pmps: usize) -> usize {
        if pmp_csr_idx / 2 == nbr_valid_pmps / 8 {
            // We are in the correct csr to filter out
            let to_filter_out: usize = ((nbr_valid_pmps / 8) + 1) * 8 - nbr_valid_pmps;

            return !0b0 >> (to_filter_out * 8);
        }
        !0b0
    }
}
