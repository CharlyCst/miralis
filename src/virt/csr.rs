//! CSR getters and setters
//!
//! This module models the CSR registers, it validates valid bit patters according to the RISC-V
//! specification.

use super::{VirtContext, VirtCsr};
use crate::arch::mie::SSIE_FILTER;
use crate::arch::pmp::pmpcfg;
use crate::arch::{hstatus, menvcfg, mie, misa, mstatus, Arch, Architecture, Csr, Register};
use crate::{debug, logger, MiralisContext, Plat, Platform};

/// A module exposing the traits to manipulate registers of a virtual context.
///
/// To get and set registers from a virtual context, first import all the traits:
///
/// ```
/// use crate::virt::traits::*;
/// ```
pub mod traits {
    pub use super::{HwRegisterContextSetter, RegisterContextGetter, RegisterContextSetter};
}

/// A trait implemented by virtual contexts to read registers.
pub trait RegisterContextGetter<R> {
    fn get(&self, register: R) -> usize;
}

/// A trait implemented by virtual contexts to write registers.
pub trait RegisterContextSetter<R> {
    fn set(&mut self, register: R, value: usize);
}

/// A trait implemented by virtual contexts to write registers whose value depends on
/// hardware capabilities..
pub trait HwRegisterContextSetter<R> {
    fn set_csr(&mut self, register: R, value: usize, mctx: &mut MiralisContext);
}

impl RegisterContextGetter<Register> for VirtContext {
    fn get(&self, register: Register) -> usize {
        // NOTE: Register x0 is never set, so always keeps a value of 0
        self.regs[register as usize]
    }
}

impl RegisterContextSetter<Register> for VirtContext {
    fn set(&mut self, register: Register, value: usize) {
        // Skip register x0
        if register == Register::X0 {
            return;
        }
        self.regs[register as usize] = value;
    }
}

impl RegisterContextGetter<Csr> for VirtContext {
    fn get(&self, register: Csr) -> usize {
        match register {
            Csr::Mhartid => self.hart_id,
            Csr::Mstatus => self.csr.mstatus,
            Csr::Misa => self.csr.misa,
            Csr::Mie => self.csr.mie,
            Csr::Mip => {
                // NOTE: here we return only the software writeable bits from the virtual context,
                // but reads the hardware normally OR the result with a special read-only bit
                // (SEIE) that comes from the hardware controller. That bit is separate from the
                // software SEIE, but it is only over possible to read the OR of those two bits.
                //
                // The issue is that the hardware bit is ignored by `csrrs` and `csrrc`, see from
                // the manual:
                //
                // > Only the software-writable SEIP bit participates in the read-modify-write
                // > sequence of a CSRRS or CSRRC instruction.
                //
                // To properly emulate this we should treat `csrrs(i)` and `csrrc(i)` differently
                // when accessing `mip`. For now we simply choose the easy solution and hide the
                // hardware bit from the virtualized firmware.
                self.csr.mip
            }
            Csr::Mtvec => self.csr.mtvec,
            Csr::Mscratch => self.csr.mscratch,
            Csr::Mvendorid => self.csr.mvendorid as usize,
            Csr::Marchid => self.csr.marchid,
            Csr::Mimpid => self.csr.mimpid,
            Csr::Pmpcfg(pmp_cfg_idx) => {
                if pmp_cfg_idx % 2 == 1 {
                    // This should not happen, as we check in the decoder for the pmp index.
                    // We return zero nontheless, because it is what the Sail implementation does,
                    // so that way we pass the model checking step.
                    log::warn!("Invalid pmpcfg {}", pmp_cfg_idx);
                    return 0;
                }
                if pmp_cfg_idx >= 2 * self.nb_pmp / 8 {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmpcfg[pmp_cfg_idx / 2]
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nb_pmp {
                    // This PMP is not emulated
                    return 0;
                }
                let pmpcfg = self.get_pmpcfg(pmp_addr_idx);
                let addr = self.csr.pmpaddr[pmp_addr_idx];
                let a1 = (pmpcfg & 0b10000) != 0;
                let g = self.pmp_grain;
                match a1 {
                    true if g >= 2 => {
                        let mask = (1 << (g - 1)) - 1;
                        addr | mask
                    }
                    false if g >= 1 => {
                        let mask = (1 << g) - 1;
                        addr & !mask
                    }
                    _ => addr,
                }
            }
            Csr::Mcycle => self.csr.mcycle,
            Csr::Minstret => self.csr.minstret,
            Csr::Mhpmcounter(n) => self.csr.mhpmcounter[n],
            Csr::Mcountinhibit => self.csr.mcountinhibit as usize,
            Csr::Mhpmevent(n) => self.csr.mhpmevent[n],
            Csr::Mcounteren => self.csr.mcounteren as usize,
            Csr::Menvcfg => self.csr.menvcfg,
            Csr::Mseccfg => self.csr.mseccfg,
            Csr::Medeleg => self.csr.medeleg,
            Csr::Mideleg => self.csr.mideleg,
            Csr::Mtinst => {
                if self.extensions.has_h_extension {
                    self.csr.mtinst
                } else {
                    panic!("Mtinst exists only in H mode")
                }
            }
            Csr::Mtval2 => {
                if self.extensions.has_h_extension {
                    self.csr.mtval2
                } else {
                    panic!("Mtval exists only in H mode")
                }
            }
            Csr::Tdata1 => todo!(),                 // TODO : normal read
            Csr::Tdata2 => todo!(),                 // TODO : normal read
            Csr::Tdata3 => todo!(),                 // TODO : normal read
            Csr::Mcontext => todo!(),               // TODO : normal read
            Csr::Dcsr => todo!(),                   // TODO : normal read
            Csr::Dpc => todo!(),                    // TODO : normal read
            Csr::Dscratch0 => todo!(),              // TODO : normal read
            Csr::Dscratch1 => todo!(),              // TODO : normal read
            Csr::Mconfigptr => self.csr.mconfigptr, // Read-only
            Csr::Tselect => !self.csr.tselect,
            Csr::Mepc => self.csr.mepc & self.pc_alignment_mask(),
            Csr::Mcause => self.csr.mcause,
            Csr::Mtval => self.csr.mtval,
            //Supervisor-level CSRs
            Csr::Sstatus => self.get(Csr::Mstatus) & mstatus::SSTATUS_FILTER,
            Csr::Sie => self.get(Csr::Mie) & mie::SIE_FILTER & self.get(Csr::Mideleg),
            Csr::Stvec => self.csr.stvec,
            Csr::Scounteren => self.csr.scounteren as usize,
            Csr::Senvcfg => self.csr.senvcfg,
            Csr::Sscratch => self.csr.sscratch,
            Csr::Sepc => self.csr.sepc & self.pc_alignment_mask(),
            Csr::Scause => self.csr.scause,
            Csr::Stval => self.csr.stval,
            Csr::Sip => self.get(Csr::Mip) & mie::SIE_FILTER & self.get(Csr::Mideleg),
            Csr::Satp => self.csr.satp,
            Csr::Scontext => self.csr.scontext,
            Csr::Stimecmp => self.csr.stimecmp,
            Csr::Hstatus => self.csr.hstatus, // TODO : Add support for H-Mode
            Csr::Hedeleg => self.csr.hedeleg,
            Csr::Hideleg => self.csr.hideleg,
            Csr::Hvip => self.csr.hvip,
            Csr::Hip => self.csr.hip,
            Csr::Hie => self.csr.hie,
            Csr::Hgeip => self.csr.hgeip,
            Csr::Hgeie => self.csr.hgeie,
            Csr::Henvcfg => self.csr.henvcfg,
            Csr::Hcounteren => self.csr.hcounteren, // TODO: Throw the virtual exeption in read
            Csr::Htimedelta => self.csr.htimedelta,
            Csr::Htval => self.csr.htval,
            Csr::Htinst => self.csr.htinst,
            Csr::Hgatp => self.csr.hgatp,
            Csr::Vsstatus => self.csr.vsstatus,
            Csr::Vsie => {
                // When bit 2 or 6 or 10 of hideleg is zero, vsip.SEIP and vsie.SEIE are read-only zeros.
                let hideleg_b_2: bool = ((self.csr.hideleg >> 2) & 0x1) != 0;
                let hideleg_b_6: bool = ((self.csr.hideleg >> 6) & 0x1) != 0;
                let hideleb_b_10: bool = ((self.csr.hideleg >> 10) & 0x1) != 0;

                if !hideleb_b_10 || !hideleg_b_6 || !hideleg_b_2 {
                    0
                } else {
                    self.csr.vsie
                }
            }
            Csr::Vstvec => self.csr.vstvec,
            Csr::Vsscratch => self.csr.vsscratch,
            Csr::Vsepc => self.csr.vsepc,
            Csr::Vscause => self.csr.vscause,
            Csr::Vstval => self.csr.vstval,
            Csr::Vsip => {
                // When bit 2 or 6 or 10 of hideleg is zero, vsip.SEIP and vsie.SEIE are read-only zeros.
                let hideleg_b_2: bool = ((self.csr.hideleg >> 2) & 0x1) != 0;
                let hideleg_b_6: bool = ((self.csr.hideleg >> 6) & 0x1) != 0;
                let hideleb_b_10: bool = ((self.csr.hideleg >> 10) & 0x1) != 0;

                if !hideleb_b_10 || !hideleg_b_6 || !hideleg_b_2 {
                    0
                } else {
                    self.csr.vsip
                }
            }
            Csr::Vsatp => self.csr.vsatp,

            // Vector extension
            Csr::Vstart => self.csr.vstart as usize,
            Csr::Vxsat => {
                if self.csr.vxsat {
                    1
                } else {
                    0
                }
            }
            Csr::Vxrm => self.csr.vxrm as usize,
            Csr::Vcsr => self.csr.vcsr as usize,
            Csr::Vl => self.csr.vl,
            Csr::Vtype => self.csr.vtype,
            Csr::Vlenb => self.csr.vlenb,

            Csr::Cycle => self.csr.mcycle,
            Csr::Time => Arch::read_csr(Csr::Time),
            Csr::Instret => self.csr.minstret,

            // Crypto extension
            // To get a true random value we defer to the hardware.
            Csr::Seed => Arch::read_csr(Csr::Seed),

            // Platform-specific CSRs
            Csr::Custom(csr) => Plat::read_custom_csr(csr),

            // Unknown
            Csr::Unknown => {
                log::warn!("Tried to access unknown CSR: {:?}", register);
                // Official specification returns 0x0 when the register is unknown
                0x0
            }
        }
    }
}

impl HwRegisterContextSetter<Csr> for VirtContext {
    fn set_csr(&mut self, register: Csr, value: usize, mctx: &mut MiralisContext) {
        let hw = &mctx.hw;
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => {
                let mut new_value = value & mstatus::MSTATUS_FILTER;

                // MPP : 11 : write legal : 0,1,3
                let mpp = (value & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET;
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    if mpp == 0 || (mpp == 1 && hw.extensions.has_s_extension) || mpp == 3 {
                        mpp
                    } else {
                        0
                    },
                );
                // SXL : 34 : read-only : MX-LEN = 64
                let mxl: usize = 2;
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SXL_OFFSET,
                    mstatus::SXL_FILTER,
                    if mctx.hw.extensions.has_s_extension {
                        mxl
                    } else {
                        0
                    },
                );
                // UXL : 32 : read-only : MX-LEN = 64
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::UXL_OFFSET,
                    mstatus::UXL_FILTER,
                    mxl,
                );

                // MPRV : 17 : write anything
                let mprv = (value & mstatus::MPRV_FILTER) >> mstatus::MPRV_OFFSET;
                let previous_mprv =
                    (self.csr.mstatus & mstatus::MPRV_FILTER) >> mstatus::MPRV_OFFSET;

                // When vMPRV transitions from 0 to 1, set up a PMP entry to protect all memory.
                // This allows catching accesses that occur with vMPRV=1, which require a special virtual access handler.
                // When vMPRV transitions back to 0, remove the protection.
                // pMPRV is never set to 1 outside of a virtual access handler.
                if mprv != previous_mprv {
                    logger::trace!("vMPRV set to {:b}", mprv);
                    if mprv != 0 {
                        mctx.pmp.set_tor(0, usize::MAX, pmpcfg::X);
                    } else {
                        mctx.pmp.set_inactive(0, usize::MAX);
                    }
                    // TODO: it seems the PMP are not yet written to hardware here,
                    // that seems like a bug to me. We should investigate.
                    // unsafe { write_pmp(&mctx.pmp).flush() };
                    unsafe { Arch::sfencevma(None, None) };
                }

                if !mctx.hw.extensions.has_s_extension || self.csr.misa & misa::S == 0 {
                    // When S mode is not active, we set a bunch of bits to 0
                    new_value &= !(mstatus::TVM_FILTER
                        | mstatus::TSR_FILTER
                        | mstatus::MXR_FILTER
                        | mstatus::SUM_FILTER
                        | mstatus::SPP_FILTER
                        | mstatus::SPIE_FILTER
                        | mstatus::SIE_FILTER);
                }

                if mctx.hw.extensions.has_zfinx {
                    // F and Zfinx are mutually exclusive
                    new_value &= !mstatus::FS_FILTER;
                }

                // We do not support changing endianness (MBE, SBE, UBE)
                new_value &= !(mstatus::MBE_FILTER | mstatus::SBE_FILTER | mstatus::UBE_FILTER);

                // No support for extensions -> XS read-only 0
                new_value &= !mstatus::XS_FILTER;

                // SD : 63 : read-only 0 (if NO FS/VS/XS)
                let fs: usize = (new_value & mstatus::FS_FILTER) >> mstatus::FS_OFFSET;
                let vs: usize = (new_value & mstatus::VS_FILTER) >> mstatus::VS_OFFSET;
                let dirty = fs == 0b11 || vs == 0b11;
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SD_OFFSET,
                    mstatus::SD_FILTER,
                    if dirty { 0b1 } else { 0b0 },
                );

                // UIE and UPIE should be zero if user-space interrupts are disabled
                if self.csr.misa & misa::N == 0 {
                    new_value &= !(mstatus::UIE_FILTER | mstatus::UPIE_FILTER);
                }

                // If the hypervisor extension is enabled, we can modify the machine previous virtualisation bit
                // This bit is similar to MPP but enables and disables virtualisation when jumping using mret
                if !self.extensions.has_h_extension {
                    new_value &= !(mstatus::GVA_FILTER | mstatus::MPV_FILTER);
                }

                self.csr.mstatus = new_value;
            }
            Csr::Misa => {} // Read only register, we don't support deactivating extensions in Miralis
            Csr::Mie => {
                if value & mie::MEIE_FILTER != 0 {
                    debug::warn_once!("MEIE bit in 'mie' is not yet supported");
                }

                self.csr.mie = hw.interrupts & value & mie::MIE_WRITE_FILTER;
            }
            Csr::Mip => {
                let value = value & hw.interrupts & mie::MIP_WRITE_FILTER;

                // If the firmware wants to read the mip register after cleaning vmip.SEIP, and we don't sync
                // vmip.SEIP with mip.SEIP, it can't know if there is an interrupt signal from the interrupt
                // controller as the CSR read will be a logical-OR of the signal and mip.SEIP (which is one)
                // so always 1. If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
                // Then, we need to synchronize vmip.SEIP with mip.SEIP.
                if (self.csr.mip ^ value) & mie::SEIE_FILTER != 0 {
                    if value & mie::SEIE_FILTER == 0 {
                        unsafe {
                            Arch::clear_csr_bits(Csr::Mip, mie::SEIE_FILTER);
                        }
                    } else {
                        unsafe {
                            Arch::set_csr_bits(Csr::Mip, mie::SEIE_FILTER);
                        }
                    }
                }

                // Keep all the non-writeable bits
                self.csr.mip = value | (self.csr.mip & !mie::MIP_WRITE_FILTER);
            }
            Csr::Mtvec => {
                match value & 0b11 {
                    // Direct mode
                    0b00 => self.csr.mtvec = value,
                    // Vector mode
                    0b01 => self.csr.mtvec = value,
                    // Reserved mode
                    _ => {
                        self.csr.mtvec = (value & !0b11) | (self.csr.mtvec & 0b11);
                    }
                }
            }
            Csr::Mscratch => self.csr.mscratch = value,
            Csr::Mvendorid => (), // Read-only
            Csr::Marchid => (),   // Read-only
            Csr::Mimpid => (),    // Read-only
            Csr::Pmpcfg(pmp_cfg_idx) => {
                let mut value = value;
                if Csr::PMP_CFG_LOCK_MASK & value != 0 {
                    debug::warn_once!("PMP lock bits are not yet supported");
                }
                if pmp_cfg_idx % 2 == 1 {
                    // Should not happen because we are in a RISCV64 setting (the decoder emmits
                    // invalid CSR instead).
                    log::warn!("Invalid pmpcfg write {}", pmp_cfg_idx);
                    return;
                } else if (pmp_cfg_idx / 2) >= self.nb_pmp / 8 {
                    // This PMP is not emulated, ignore changes
                    return;
                }

                // Legalize individual pmpcfg entries
                for idx in 0..8 {
                    let offset = idx * 8; // Bit offset

                    // W = 1 & R = 0 is reserved
                    if (value >> offset) & 0b11 == 0b10 {
                        value &= !(0b111 << offset);
                    }

                    // NA4 can not be selected if the PMP grain G >= 1
                    if self.pmp_grain >= 1 && (value >> offset) & 0b11000 == 0b10000 {
                        value &= !(0b11000 << offset);
                    }
                }

                self.csr.pmpcfg[pmp_cfg_idx / 2] = value
                    & Csr::PMP_CFG_LEGAL_MASK
                    & VirtCsr::get_pmp_cfg_filter(pmp_cfg_idx, self.nb_pmp);
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= mctx.hw.available_reg.nb_pmp {
                    // This PMP is not emulated, ignore
                    return;
                }
                self.csr.pmpaddr[pmp_addr_idx] = Csr::PMP_ADDR_LEGAL_MASK & value;
            }
            Csr::Mcycle => self.csr.mcycle = value,
            Csr::Minstret => self.csr.minstret = value,
            Csr::Mhpmcounter(_counter_idx) => (), // Read-only 0
            Csr::Mcountinhibit => {
                let mask = 0b101; // We do not support counters for now
                self.csr.mcountinhibit = (value & mask) as u32;
            }
            Csr::Mhpmevent(_event_idx) => (), // Read-only 0
            Csr::Mcounteren => {
                // Only show IR, TM and CY (for cycle, time and instret counters)
                let mask = 0b111; // We do not support counters beyond basic ones for now
                self.csr.mcounteren = (value & mask) as u32
            }
            Csr::Menvcfg => {
                let mut mask: usize = menvcfg::ALL;

                // Filter valid values based on implemented extensions.
                if !mctx.hw.extensions.has_sstc_extension {
                    mask &= !menvcfg::STCE_FILTER; // Hardwire STCE to 0 if Sstc is disabled
                }
                if !mctx.hw.extensions.has_zicbom_extension {
                    mask &= !(menvcfg::CBIE_FILTER | menvcfg::CBCFE_FILTER);
                }
                if !mctx.hw.extensions.has_zicboz_extension {
                    mask &= !menvcfg::CBZE_FILTER;
                }

                self.csr.menvcfg = value & mask;
                mctx.hw.extensions.is_sstc_enabled = self.csr.menvcfg & menvcfg::STCE_FILTER != 0;
            }
            Csr::Mseccfg => self.csr.mseccfg = value,
            Csr::Mconfigptr => (), // Read-only
            Csr::Medeleg => self.csr.medeleg = value & !(1 << 11),
            Csr::Mideleg => {
                self.csr.mideleg = (value & hw.interrupts & !mie::MIDELEG_READ_ONLY_ZERO)
                    | mie::MIDELEG_READ_ONLY_ONE;
            }
            Csr::Mtinst => {
                if mctx.hw.extensions.has_h_extension {
                    self.csr.mtinst = value
                } else {
                    panic!("Mtinst exists only in H mode")
                }
            } // TODO : Can only be written automatically by the hardware on a trap, this register should not exist in a system without hypervisor extension
            Csr::Mtval2 => {
                if mctx.hw.extensions.has_h_extension {
                    self.csr.mtval2 = value
                } else {
                    panic!("Mtval2 exists only in H mode")
                }
            } // TODO : Must be able to hold 0 and may hold an arbitrary number of 2-bit-shifted guest physical addresses, written alongside mtval, this register should not exist in a system without hypervisor extension
            Csr::Tselect => {
                self.csr.tselect = value;
            }
            Csr::Tdata1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata2 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata3 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mcontext => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dcsr => todo!(),   // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dpc => todo!(),    // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch0 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                if hw.extensions.has_c_extension {
                    self.csr.mepc = value & !0b1
                } else {
                    self.csr.mepc = value & !0b11
                }
            }
            Csr::Mcause => self.csr.mcause = value,
            Csr::Mtval => self.csr.mtval = value,
            //Supervisor-level CSRs
            Csr::Sstatus => {
                // Clear sstatus bits
                let mstatus = self.get(Csr::Mstatus) & !mstatus::SSTATUS_FILTER;
                // Set sstatus bits to new value
                self.set_csr(
                    Csr::Mstatus,
                    mstatus | (value & mstatus::SSTATUS_FILTER),
                    mctx,
                );
            }
            Csr::Sie => {
                // Only delegated interrupts can be enabled through `sie`
                let mideleg = self.get(Csr::Mideleg);
                self.csr.mie = (self.csr.mie & !mideleg) | (mideleg & value);
            }
            Csr::Stvec => {
                match value & 0b11 {
                    // Direct mode
                    0b00 => self.csr.stvec = value,
                    // Vector mode
                    0b01 => self.csr.stvec = value,
                    // Reserved mode
                    _ => {
                        self.csr.stvec = (value & !0b11) | (self.csr.stvec & 0b11);
                    }
                }
            }
            Csr::Scounteren => {
                // Only show IR, TM and CY (for cycle, time and instret counters)
                let mask = 0b111; // We do not support counters beyond basic ones for now
                self.csr.scounteren = (value & mask) as u32
            }
            Csr::Senvcfg => {
                let mut mask = menvcfg::FIOM_FILTER
                    | menvcfg::CBIE_FILTER
                    | menvcfg::CBZE_FILTER
                    | menvcfg::CBCFE_FILTER;

                // Filter valid values based on implemented extensions.
                if !mctx.hw.extensions.has_zicbom_extension {
                    mask &= !(menvcfg::CBIE_FILTER | menvcfg::CBCFE_FILTER);
                }
                if !mctx.hw.extensions.has_zicboz_extension {
                    mask &= !menvcfg::CBZE_FILTER;
                }

                self.csr.senvcfg = value & mask
            }
            Csr::Sscratch => self.csr.sscratch = value,
            Csr::Sepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                if hw.extensions.has_c_extension {
                    self.csr.sepc = value & !0b1
                } else {
                    self.csr.sepc = value & !0b11
                }
            }
            Csr::Scause => self.csr.scause = value,
            Csr::Stval => self.csr.stval = value,
            Csr::Sip => {
                if self.csr.mideleg & SSIE_FILTER != 0 {
                    self.csr.mip = (self.csr.mip & !SSIE_FILTER) | (SSIE_FILTER & value);
                }
            }
            Csr::Satp => {
                let satp_mode = (value >> 60) & 0b1111;
                match satp_mode {
                    // Sbare mode
                    0b0000 => self.csr.satp = value,
                    // Sv39 mode
                    0b1000 => self.csr.satp = value,
                    // Sv48 mode
                    0b1001 => {} // Not yet supported
                    // No mode
                    _ => { /* Nothing to change */ }
                }
            }
            Csr::Scontext => (), // TODO: No information from the specification currently
            Csr::Stimecmp => self.csr.stimecmp = value,
            Csr::Hstatus => {
                let mut value = value;

                // VSXL is a read only two as we only support 64 bit mode
                const VSXL: usize = 2;
                VirtCsr::set_csr_field(
                    &mut value,
                    hstatus::VSXL_OFFSET,
                    hstatus::VSXL_FILTER,
                    VSXL,
                );

                if !mctx.hw.extensions.has_s_extension {
                    // VTSR is read only if S-mode is not present
                    VirtCsr::set_csr_field(
                        &mut value,
                        hstatus::VTSR_OFFSET,
                        hstatus::VTSR_FILTER,
                        0,
                    );
                    // VTVM is read only if S-mode is not present
                    VirtCsr::set_csr_field(
                        &mut value,
                        hstatus::VTVM_OFFSET,
                        hstatus::VTVM_FILTER,
                        0,
                    );
                    // VTW is read only if H mode is the lowest priviledge mode
                    // and U-mode must exist in Miralis
                    VirtCsr::set_csr_field(&mut value, hstatus::VTW_FILTER, hstatus::VTW_FILTER, 0);
                }

                // We don't implement the feature as it is a very niche one
                if value & hstatus::VSBE_FILTER != 0 {
                    todo!("VSBE field set to 1 isn't implemented, please implement it")
                }

                self.csr.hstatus = value
            }
            Csr::Hedeleg => {
                let write_hedeleg_mask: usize = !((0b111 << 9) | (0b1111 << 20));
                self.csr.hedeleg = value & write_hedeleg_mask;
            }
            Csr::Hideleg => {
                let write_hideleg_mask: usize = !((12 << 1) | (9 << 1) | (5 << 1) | (1 << 1));
                self.csr.hideleg = value & write_hideleg_mask;
            }
            Csr::Hvip => {
                let write_hvip_mask: usize =
                    !((0b11111 << 11) | (0b111 << 7) | (0b111 << 3) | (0b11));
                self.csr.hvip = value & write_hvip_mask;
            }
            Csr::Hip => {
                let write_hip_mask: usize =
                    !((0b111 << 13) | (0b1 << 11) | (0b111 << 7) | (0b111 << 3) | (0b11));
                self.csr.hip = value & write_hip_mask;
            }
            Csr::Hie => {
                let write_hie_mask: usize =
                    !((0b111 << 13) | (0b1 << 11) | (0b111 << 7) | (0b111 << 3) | (0b11));
                self.csr.hie = value & write_hie_mask;
            }
            Csr::Hgeip => {} // Read-only register
            Csr::Hgeie => {
                self.csr.hgeie = value;
                // Last bit is always 0
                self.csr.hgeie &= !1;
            }
            Csr::Henvcfg => self.csr.henvcfg = value,
            Csr::Hcounteren => self.csr.hcounteren = value,
            Csr::Htimedelta => self.csr.htimedelta = value,
            Csr::Htval => self.csr.htval = value,
            Csr::Htinst => self.csr.htinst = value,
            Csr::Hgatp => {
                self.csr.hgatp = value & !(0b11 << 58);
            }
            Csr::Vsstatus => self.csr.vsstatus = value,
            Csr::Vsie => {
                let write_vsie_mask: usize =
                    !((0b111111 << 10) | (0b111 << 6) | (0b111 << 2) | (0b1));
                self.csr.vsie = value & write_vsie_mask
            }
            Csr::Vstvec => self.csr.vstvec = value,
            Csr::Vsscratch => self.csr.vsscratch = value,
            Csr::Vsepc => self.csr.vsepc = value,
            Csr::Vscause => self.csr.vscause = value,
            Csr::Vstval => self.csr.vstval = value,
            Csr::Vsip => {
                let write_vsip_mask: usize =
                    !((0b111111 << 10) | (0b111 << 6) | (0b111 << 2) | (0b1));
                self.csr.vsip = value & write_vsip_mask
            }
            Csr::Vsatp => self.csr.vsatp = value,

            // Vector extension
            Csr::Vstart => self.csr.vstart = (value & 0xff) as u16,
            Csr::Vxsat => self.csr.vxsat = (value & 0x1) != 0,
            Csr::Vxrm => self.csr.vxrm = (value & 0b11) as u8,
            Csr::Vcsr => self.csr.vcsr = (value & 0b111) as u8,
            Csr::Vl => self.csr.vl = value,
            Csr::Vtype => self.csr.vtype = value,
            Csr::Vlenb => self.csr.vlenb = value,

            Csr::Cycle => (),   // Read only register
            Csr::Time => (),    // Read only register
            Csr::Instret => (), // Read only register

            // Crypto extension
            Csr::Seed => (), // Read only register

            // Platform-specific CSRs
            Csr::Custom(csr) => Plat::write_custom_csr(csr, value),

            // Unknown
            // Specification ingnores the write to a non-existing register
            Csr::Unknown => log::warn!("Tried to access unknown CSR: {:?}", register),
        }
    }
}

/// Forward RegisterContextGetter implementation for register references
impl<'a, R> RegisterContextGetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: RegisterContextGetter<R>,
{
    #[inline]
    fn get(&self, register: &'a R) -> usize {
        self.get(*register)
    }
}

/// Forward RegisterContextSetter implementation for register references
impl<'a, R> RegisterContextSetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: RegisterContextSetter<R>,
{
    #[inline]
    fn set(&mut self, register: &'a R, value: usize) {
        self.set(*register, value)
    }
}

/// Forward HwCsrRegisterContextSetter implementation for register references
impl<'a, R> HwRegisterContextSetter<&'a R> for VirtContext
where
    R: Copy,
    VirtContext: HwRegisterContextSetter<R>,
{
    #[inline]
    fn set_csr(&mut self, register: &'a R, value: usize, mctx: &mut MiralisContext) {
        self.set_csr(*register, value, mctx)
    }
}

// ———————————————————————————————— Helpers ————————————————————————————————— //

impl VirtContext {
    /// Return the PMP configuration for a given PMP index.
    pub fn get_pmpcfg(&self, index: usize) -> u8 {
        let reg_idx = index / 8;
        let inner_idx = index % 8;
        let reg = self.csr.pmpcfg[reg_idx];
        let cfg = (reg >> (inner_idx * 8)) & 0xff;
        cfg as u8
    }
}
