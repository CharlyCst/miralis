//! Firmware Virtualisation

use miralis_core::abi;

use crate::arch::mstatus::{MBE_FILTER, SBE_FILTER, UBE_FILTER};
use crate::arch::pmp::pmpcfg;
use crate::arch::pmp::pmpcfg::NO_PERMISSIONS;
use crate::arch::{
    hstatus, mie, misa, mstatus, mtvec, parse_mpp_return_mode, satp, Arch, Architecture, Csr,
    ExtensionsCapability, MCause, Mode, Register, TrapInfo,
};
use crate::benchmark::Benchmark;
use crate::config::DELEGATE_PERF_COUNTER;
use crate::decoder::Instr;
use crate::device::VirtDevice;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{Policy, PolicyModule};
use crate::utils::sign_extend;
use crate::{debug, device, utils};

/// The execution mode, either virtualized firmware or native payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Virtualized virmware, running in U-mode.
    Firmware,
    /// Native payload, running in U or S-mode.
    Payload,
}

/// The context of a virtual firmware.
#[derive(Debug)]
#[repr(C)]
pub struct VirtContext {
    /// Stack pointer of the host, used to restore context on trap.
    host_stack: usize,
    /// Basic registers
    pub(crate) regs: [usize; 32],
    /// Program Counter
    pub(crate) pc: usize,
    /// Information on the trap that ocurred, used to handle traps
    pub(crate) trap_info: TrapInfo,
    /// Virtual Control and Status Registers
    pub(crate) csr: VirtCsr,
    /// Current privilege mode
    pub(crate) mode: Mode,
    /// Number of virtual PMPs
    pub(crate) nb_pmp: usize,
    /// Availables RISC-V extensions
    pub(crate) extensions: ExtensionsCapability,
    /// Hart ID
    pub(crate) hart_id: usize,
    /// Number of exists to Miralis
    pub(crate) nb_exits: usize,
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
            },
            pc: 0,
            mode: Mode::M,
            nb_pmp: nb_pmp_registers_left,
            trap_info: TrapInfo {
                mepc: 0,
                mstatus: 0,
                mcause: 0,
                mip: 0,
                mtval: 0,
            },
            nb_exits: 0,
            hart_id,
            extensions: available_extension,
        }
    }
}

/// Control and Status Registers (CSR) for a virtual firmware.
#[derive(Debug)]
#[repr(C)]
pub struct VirtCsr {
    pub misa: usize,
    pub mie: usize,
    pub mip: usize,
    pub mtvec: usize,
    pub mvendorid: usize,
    pub marchid: usize,
    pub mimpid: usize,
    pub mcycle: usize,
    pub minstret: usize,
    pub mscratch: usize,
    pub mcountinhibit: usize,
    pub mcounteren: usize,
    pub menvcfg: usize,
    pub mseccfg: usize,
    pub mcause: usize,
    pub mepc: usize,
    pub mtval: usize,
    pub mtval2: usize,
    pub mstatus: usize,
    pub mtinst: usize,
    pub mconfigptr: usize,
    pub stvec: usize,
    pub scounteren: usize,
    pub senvcfg: usize,
    pub sscratch: usize,
    pub sepc: usize,
    pub scause: usize,
    pub stval: usize,
    pub satp: usize,
    pub scontext: usize,
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
}

impl VirtCsr {
    pub fn set_csr_field(csr: &mut usize, offset: usize, filter: usize, value: usize) {
        debug_assert_eq!((value << offset) & !filter, 0, "Invalid set_csr_field");
        // Clear field
        *csr &= !filter;
        // Set field
        *csr |= value << offset;
    }

    /// Returns the mask of valid bit for the given PMP configuration register.
    pub fn get_pmp_cfg_filter(pmp_csr_idx: usize, nbr_valid_pmps: usize) -> usize {
        if pmp_csr_idx == nbr_valid_pmps / 8 {
            // We are in the correct csr to filter out
            let to_filter_out: usize = ((nbr_valid_pmps / 8) + 1) * 8 - nbr_valid_pmps;

            return !0b0 >> (to_filter_out * 8);
        }
        !0b0
    }
}

impl VirtContext {
    fn emulate_privileged_instr(&mut self, instr: &Instr, mctx: &mut MiralisContext) {
        match instr {
            Instr::Wfi => {
                // NOTE: for now there is no safeguard which guarantees that we will eventually get
                // an interrupt, so the firmware might be able to put the core in perpetual sleep
                // state.

                // Set mie to csr.mie, even if mstatus.MIE bit is cleared.
                unsafe {
                    Arch::write_csr(Csr::Mie, self.csr.mie);
                }

                Arch::wfi();
                self.pc += 4;
            }
            Instr::Csrrw { csr, .. }
            | Instr::Csrrs { csr, .. }
            | Instr::Csrrc { csr, .. }
            | Instr::Csrrwi { csr, .. }
            | Instr::Csrrsi { csr, .. }
            | Instr::Csrrci { csr, .. }
                if csr.is_unknown() =>
            {
                self.emulate_jump_trap_handler();
            }
            Instr::Csrrw { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set_csr(csr, self.get(rs1), mctx);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrs { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp | self.get(rs1), mctx);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrwi { csr, rd, uimm } => {
                self.set(rd, self.get(csr));
                self.set_csr(csr, *uimm, mctx);
                self.pc += 4;
            }
            Instr::Csrrsi { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp | uimm, mctx);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrc { csr, rd, rs1 } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp & !self.get(rs1), mctx);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Csrrci { csr, rd, uimm } => {
                let tmp = self.get(csr);
                self.set_csr(csr, tmp & !uimm, mctx);
                self.set(rd, tmp);
                self.pc += 4;
            }
            Instr::Mret => {
                match parse_mpp_return_mode(self.csr.mstatus) {
                    Mode::M => {
                        log::trace!("mret to m-mode to {:x}", self.trap_info.mepc);
                        // Mret is jumping back to machine mode, do nothing
                    }
                    Mode::S if mctx.hw.extensions.has_s_extension => {
                        log::trace!("mret to s-mode with MPP to {:x}", self.trap_info.mepc);
                        // Mret is jumping to supervisor mode, the runner is the guest OS
                        self.mode = Mode::S;

                        VirtCsr::set_csr_field(
                            &mut self.csr.mstatus,
                            mstatus::MPRV_OFFSET,
                            mstatus::MPRV_FILTER,
                            0,
                        );
                    }
                    Mode::U => {
                        log::trace!("mret to u-mode with MPP");
                        // Mret is jumping to user mode, the runner is the guest OS
                        self.mode = Mode::U;

                        VirtCsr::set_csr_field(
                            &mut self.csr.mstatus,
                            mstatus::MPRV_OFFSET,
                            mstatus::MPRV_FILTER,
                            0,
                        );
                    }
                    _ => {
                        panic!(
                            "MRET is not going to M/S/U mode: {} with MPP {:x}",
                            self.csr.mstatus,
                            (self.csr.mstatus & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET
                        );
                    }
                }
                // Modify mstatus
                // ONLY WITH HYPERVISOR EXTENSION : MPV = 0,
                if false {
                    VirtCsr::set_csr_field(
                        &mut self.csr.mstatus,
                        mstatus::MPV_OFFSET,
                        mstatus::MPV_FILTER,
                        0,
                    );
                }

                // MIE = MPIE, MPIE = 1, MPRV = 0
                let mpie = (self.csr.mstatus & mstatus::MPIE_FILTER) >> mstatus::MPIE_OFFSET;

                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPIE_OFFSET,
                    mstatus::MPIE_FILTER,
                    1,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MIE_OFFSET,
                    mstatus::MIE_FILTER,
                    mpie,
                );
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    0,
                );

                // Jump back to firmware
                self.pc = self.csr.mepc;
            }
            Instr::Sfencevma { rs1, rs2 } => unsafe {
                let vaddr = match rs1 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                let asid = match rs2 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                Arch::sfencevma(vaddr, asid);
                self.pc += 4;
            },
            Instr::Hfencegvma { rs1, rs2 } => unsafe {
                let vaddr = match rs1 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                let asid = match rs2 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                Arch::hfencegvma(vaddr, asid);
                self.pc += 4;
            },
            Instr::Hfencevvma { rs1, rs2 } => unsafe {
                let vaddr = match rs1 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                let asid = match rs2 {
                    Register::X0 => None,
                    reg => Some(self.get(reg)),
                };
                Arch::hfencevvma(vaddr, asid);
                self.pc += 4;
            },
            _ => todo!(
                "Instruction not yet implemented: {:?} {:x} {:x}",
                instr,
                self.csr.mepc,
                self.csr.mtval
            ),
        }
    }

    /// Handles a load instruction.
    ///
    /// Calculates the memory address, reads the value from the device,
    /// sign-extends (normal load) or zero-extends (unsigned load) it to 64 bits if necessary,
    /// applies a mask and writes the value to the device.
    ///
    /// - Normal load&store instructions are 4 bytes long.
    /// - The immediate (`imm`) value can be positive or negative.
    /// - Compressed load&store instructions are 2 bytes long.
    /// - The immediate (`imm`) value is always positive.
    fn handle_load(&mut self, device: &VirtDevice, instr: &Instr) {
        match instr {
            Instr::Load {
                rd,
                rs1,
                imm,
                len,
                is_compressed,
                is_unsigned,
            } => {
                let address = utils::calculate_addr(self.get(*rs1), *imm);
                let offset = address - device.start_addr;

                match device.device_interface.read_device(offset, *len, self) {
                    Ok(value) => {
                        let value = if !is_unsigned {
                            sign_extend(value, *len)
                        } else {
                            value
                        };

                        self.set(*rd, value);
                        self.pc += if *is_compressed { 2 } else { 4 };
                    }
                    Err(err) => panic!("Error reading {}: {}", device.name, err),
                }
            }
            _ => panic!("Not a load instruction in a load handler"),
        }
    }

    /// Handles a store instruction.
    ///
    /// Calculates the memory address and writes the value
    /// to the device (after applying a mask to prevent overflow).
    fn handle_store(&mut self, device: &VirtDevice, instr: &Instr) {
        match instr {
            Instr::Store {
                rs2,
                rs1,
                imm,
                len,
                is_compressed,
            } => {
                let address = utils::calculate_addr(self.get(*rs1), *imm);
                let offset = address - device.start_addr;

                let value = self.get(*rs2);

                let mask = if len.to_bits() < usize::BITS as usize {
                    (1 << len.to_bits()) - 1
                } else {
                    usize::MAX
                };

                if value > mask {
                    log::warn!(
                        "Value {} exceeds allowed length {}. Trimming to fit.",
                        value,
                        len.to_bits()
                    );
                }

                match device
                    .device_interface
                    .write_device(offset, *len, value & mask, self)
                {
                    Ok(()) => {
                        // Update the program counter (pc) based on compression
                        self.pc += if *is_compressed { 2 } else { 4 };
                    }
                    Err(err) => panic!("Error writing {}: {}", device.name, err),
                }
            }
            _ => panic!("Not a store instruction in a store handler"),
        }
    }

    pub fn handle_device_access_fault(&mut self, instr: &Instr, device: &VirtDevice) {
        match instr {
            Instr::Load { .. } => self.handle_load(device, instr),
            Instr::Store { .. } => self.handle_store(device, instr),
            _ => todo!("Instruction not yet implemented: {:?}", instr),
        }
    }

    /// Check if an interrupt should be injected in virtual M-mode.
    ///
    /// If an interrupt is injected, jumps to the firmware trap handler.
    pub fn check_and_inject_interrupts(&mut self) {
        if self.csr.mstatus & mstatus::MIE_FILTER == 0 && self.mode == Mode::M {
            // Interrupts are disabled while in M-mode if mstatus.MIE is 0
            return;
        }
        let Some(next_int) = get_next_interrupt(self.csr.mie, self.csr.mip, self.csr.mideleg)
        else {
            // No enabled interrupt pending
            return;
        };

        // Update Mstatus to match the semantic of a trap
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            self.mode.to_bits(),
        );
        let mpie = (self.csr.mstatus & mstatus::MIE_FILTER) >> mstatus::MIE_OFFSET;
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MPIE_OFFSET,
            mstatus::MPIE_FILTER,
            mpie,
        );
        VirtCsr::set_csr_field(
            &mut self.csr.mstatus,
            mstatus::MIE_OFFSET,
            mstatus::MIE_FILTER,
            0,
        );

        let mcause = next_int | (1 << (usize::BITS - 1));
        self.csr.mcause = mcause;
        self.csr.mepc = self.pc;
        self.csr.mtval = 0;
        self.mode = Mode::M;
        self.set_pc_to_mtvec();
    }

    pub fn emulate_jump_trap_handler(&mut self) {
        // We are now emulating a trap, registers need to be updated
        log::trace!("Emulating jump to trap handler");
        self.csr.mcause = self.trap_info.mcause;
        self.csr.mstatus = self.trap_info.mstatus;
        self.csr.mtval = self.trap_info.mtval;
        self.csr.mepc = self.trap_info.mepc;
        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        // MSIE and MEIE could not be set by payload so it should be 0. The real value is read from hardware when
        // the firmware want to read virtual mip.
        //
        // We also preserve the virtualized interrupt bits from the virtual mip, as those are pure
        // software and might not match the physical mip.
        let hw_mip_bits = self.trap_info.mip & !(mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        let sw_mip_bits = self.csr.mip & (mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        self.csr.mip = hw_mip_bits | sw_mip_bits;

        match self.mode {
            Mode::M => {
                // Modify mstatus: previous privilege mode is machine = 3
                VirtCsr::set_csr_field(
                    &mut self.csr.mstatus,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    Mode::M.to_bits(),
                );
            }
            _ => {
                // No need to modify mstatus: MPP is correct
                self.mode = Mode::M;
            }
        }

        // Go to firmware trap handler
        self.set_pc_to_mtvec();
    }

    /// Set the program counter (PC) to `mtvec`, amulating a jump to the trap handler.
    ///
    /// This function checks the `mcause` CSR to select the right entry point if `mtvec` is in
    /// vectored more. Therefore it assumes `mcause` has been configured prior to calling this
    /// function.
    fn set_pc_to_mtvec(&mut self) {
        self.pc = match mtvec::get_mode(self.csr.mtvec) {
            // If Direct mode: just jump to BASE directly
            mtvec::Mode::Direct => self.csr.mtvec & mtvec::BASE_FILTER,
            // If Vectored mode: if synchronous exception, jump to the BASE directly
            // else, jump to BASE + 4 * cause
            mtvec::Mode::Vectored => {
                if MCause::is_interrupt(MCause::new(self.csr.mcause)) {
                    (self.csr.mtvec & mtvec::BASE_FILTER)
                        + 4 * MCause::cause_number(self.csr.mcause)
                } else {
                    self.csr.mtvec & mtvec::BASE_FILTER
                }
            }
        }
    }

    /// Handles a machine timer interrupt
    ///
    /// TODO: for now we assume that all M-mode timer interrupts are issued from the
    /// firmware (in-band interrupts), so we just set the bit in `vmip`.
    /// In the future we might want to support timer interrupts for Miralis' own purpose
    /// (out-of-band interrupts). Once we add such support we should disambiguate
    /// interrupts here.
    fn handle_machine_timer_interrupt(&mut self, mctx: &mut MiralisContext) {
        let mut clint = Plat::get_clint().lock();
        clint
            .write_mtimecmp(mctx.hw.hart, usize::MAX)
            .expect("Failed to write mtimecmp");
        drop(clint); // Release the lock early

        self.csr.mip |= mie::MTIE_FILTER;
    }

    /// Handles a machine software interrupt trap
    fn handle_machine_software_interrupt(
        &mut self,
        mctx: &mut MiralisContext,
        policy: &mut Policy,
    ) {
        // Clear the interrupt
        let mut clint = Plat::get_clint().lock();
        clint
            .write_msip(mctx.hw.hart, 0)
            .expect("Failed to write msip");
        drop(clint); // Release the lock early

        // Check if a virtual MSI is pending
        let vclint = Plat::get_vclint();
        if vclint.get_vmsi(self.hart_id) {
            self.csr.mip |= mie::MSIE_FILTER;
        } else {
            self.csr.mip &= !mie::MSIE_FILTER;
        }

        // Check if a policy MSI is pending
        if vclint.get_policy_msi(self.hart_id) {
            vclint.clear_policy_msi(self.hart_id);
            policy.on_interrupt(self, mctx);
        }
    }

    /// Handle the trap coming from the firmware
    pub fn handle_firmware_trap(&mut self, mctx: &mut MiralisContext, policy: &mut Policy) {
        if policy.trap_from_firmware(mctx, self).overwrites() {
            log::trace!("Catching trap in the policy module");
            return;
        }

        let cause = self.trap_info.get_cause();
        match cause {
            MCause::EcallFromUMode if policy.ecall_from_firmware(mctx, self).overwrites() => {
                // Nothing to do, the policy module handles those ecalls
                log::trace!("Catching E-call from firmware in the policy module");
            }
            MCause::EcallFromUMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                self.handle_ecall()
            }
            MCause::EcallFromUMode => {
                todo!("ecall is not yet supported for EID other than Miralis ABI");
            }
            MCause::EcallFromSMode => {
                panic!("Firmware should not be able to come from S-mode");
            }
            MCause::IllegalInstr => {
                let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                let instr = mctx.decode(instr);
                log::trace!("Faulting instruction: {:?}", instr);
                self.emulate_privileged_instr(&instr, mctx);
            }
            MCause::Breakpoint => {
                self.emulate_jump_trap_handler();
            }
            MCause::StoreAccessFault | MCause::LoadAccessFault => {
                // PMP faults
                if let Some(device) =
                    device::find_matching_device(self.trap_info.mtval, &mctx.devices)
                {
                    let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                    let instr = mctx.decode(instr);
                    log::trace!(
                        "Accessed devices: {} | With instr: {:?}",
                        device.name,
                        instr
                    );
                    self.handle_device_access_fault(&instr, device);
                } else if (self.csr.mstatus & mstatus::MPRV_FILTER) >> mstatus::MPRV_OFFSET == 1 {
                    // TODO: make sure virtual address does not get around PMP protection
                    let instr = unsafe { Arch::get_raw_faulting_instr(&self.trap_info) };
                    let instr = mctx.decode(instr);
                    log::trace!(
                        "Access fault {:x?} with a virtual address: 0x{:x}",
                        &instr,
                        self.trap_info.mtval
                    );
                    unsafe {
                        Arch::handle_virtual_load_store(instr, self);
                    }
                } else {
                    log::trace!(
                        "No matching device found for address: {:x}",
                        self.trap_info.mtval
                    );
                    self.emulate_jump_trap_handler();
                }
            }
            MCause::InstrAccessFault => {
                log::trace!("Instruction access fault: {:x?}", self.trap_info);
                self.emulate_jump_trap_handler();
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                log::info!("Machine soft int");
                self.handle_machine_software_interrupt(mctx, policy);
            }
            MCause::MachineExternalInt => {
                todo!("Virtualize machine external interrupt")
            }
            MCause::LoadAddrMisaligned
            | MCause::StoreAddrMisaligned
            | MCause::InstrAddrMisaligned => self.emulate_jump_trap_handler(),
            _ => {
                if cause.is_interrupt() {
                    // TODO : For now, only care for MTIP bit
                    todo!(
                        "Other interrupts are not yet implemented {:?} at {:x}",
                        cause,
                        self.trap_info.mepc
                    );
                } else {
                    // TODO : Need to match other traps
                    todo!(
                        "Other traps are not yet implemented {:?} at {:x}",
                        cause,
                        self.trap_info.mepc
                    );
                }
            }
        }
    }

    /// Handle the trap coming from the payload
    pub fn handle_payload_trap(&mut self, mctx: &mut MiralisContext, policy: &mut Policy) {
        // Update the current mode
        self.mode = parse_mpp_return_mode(self.trap_info.mstatus);

        if policy.trap_from_payload(mctx, self).overwrites() {
            log::trace!("Catching trap in the policy module");
            return;
        }

        // Handle the exit.
        // We only care about ecalls and virtualized interrupts.
        match self.trap_info.get_cause() {
            MCause::EcallFromSMode if policy.ecall_from_payload(mctx, self).overwrites() => {
                // Nothing to do, the Policy module handles those ecalls
                log::trace!("Catching E-call from payload in the policy module");
            }
            MCause::EcallFromSMode if self.get(Register::X17) == abi::MIRALIS_EID => {
                self.handle_ecall()
            }
            MCause::MachineTimerInt => {
                self.handle_machine_timer_interrupt(mctx);
            }
            MCause::MachineSoftInt => {
                self.handle_machine_software_interrupt(mctx, policy);
            }
            _ => self.emulate_jump_trap_handler(),
        }
    }

    /// Ecalls may come from firmware or payload, resulting in different handling.
    fn handle_ecall(&mut self) {
        let fid = self.get(Register::X16);
        match fid {
            abi::MIRALIS_FAILURE_FID => {
                log::error!("Firmware or payload panicked!");
                log::error!("  pc:    0x{:x}", self.pc);
                log::error!("  exits: {}", self.nb_exits);
                unsafe { debug::log_stack_usage() };
                Plat::exit_failure();
            }
            abi::MIRALIS_SUCCESS_FID => {
                log::info!("Success!");
                log::info!("Number of exits: {}", self.nb_exits);
                unsafe { debug::log_stack_usage() };
                Plat::exit_success();
            }
            abi::MIRALIS_LOG_FID => {
                let log_level = self.get(Register::X10);
                let addr = self.get(Register::X11);
                let size = self.get(Register::X12);

                // TODO: add proper validation that this memory range belongs to the
                // payload
                let bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, size) };
                let message =
                    core::str::from_utf8(bytes).unwrap_or("note: invalid message, not utf-8");
                match log_level {
                    abi::log::MIRALIS_ERROR => log::error!("> {}", message),
                    abi::log::MIRALIS_WARN => log::warn!("> {}", message),
                    abi::log::MIRALIS_INFO => log::info!("> {}", message),
                    abi::log::MIRALIS_DEBUG => log::debug!("> {}", message),
                    abi::log::MIRALIS_TRACE => log::trace!("> {}", message),
                    _ => {
                        log::info!("Miralis log SBI call with invalid level: {}", log_level)
                    }
                }

                // For now we don't return error code or the lenght written
                self.set(Register::X10, 0);
                self.set(Register::X11, 0);
                self.pc += 4;
            }
            abi::MIRALIS_BENCHMARK_FID => {
                Benchmark::record_counters();
                Plat::exit_success();
            }
            _ => panic!("Invalid Miralis FID: 0x{:x}", fid),
        }
    }

    /// Loads the S-mode CSR registers into the physical registers configures M-mode registers for
    /// payload execution.
    pub unsafe fn switch_from_firmware_to_payload(&mut self, mctx: &mut MiralisContext) {
        let mut mstatus = self.csr.mstatus; // We need to set the next mode bits before mret
        VirtCsr::set_csr_field(
            &mut mstatus,
            mstatus::MPP_OFFSET,
            mstatus::MPP_FILTER,
            self.mode.to_bits(),
        );

        if mctx.hw.available_reg.senvcfg {
            Arch::write_csr(Csr::Senvcfg, self.csr.senvcfg);
        }

        if mctx.hw.available_reg.menvcfg {
            Arch::write_csr(Csr::Menvcfg, self.csr.menvcfg);
        }

        Arch::write_csr(Csr::Mstatus, mstatus & !mstatus::MIE_FILTER);
        Arch::write_csr(Csr::Mideleg, self.csr.mideleg);
        Arch::write_csr(Csr::Medeleg, self.csr.medeleg);
        Arch::write_csr(Csr::Mcounteren, self.csr.mcounteren);

        // NOTE: `mip` mut be set _after_ `menvcfg`, because `menvcfg` might change which bits in
        // `mip` are writeable. For more information see the Sstc extension specification.
        Arch::write_csr(Csr::Mip, self.csr.mip);
        Arch::write_csr(Csr::Mie, self.csr.mie);

        // If S extension is present - save the registers
        if mctx.hw.extensions.has_s_extension {
            Arch::write_csr(Csr::Stvec, self.csr.stvec);
            Arch::write_csr(Csr::Scounteren, self.csr.scounteren);
            Arch::write_csr(Csr::Satp, self.csr.satp);
            Arch::write_csr(Csr::Sscratch, self.csr.sscratch);
            Arch::write_csr(Csr::Sepc, self.csr.sepc);
            Arch::write_csr(Csr::Scause, self.csr.scause);
            Arch::write_csr(Csr::Stval, self.csr.stval);
        }

        // If H extension is present - save the registers
        if mctx.hw.extensions.has_h_extension {
            Arch::write_csr(Csr::Hstatus, self.csr.hstatus);
            Arch::write_csr(Csr::Hedeleg, self.csr.hedeleg);
            Arch::write_csr(Csr::Hideleg, self.csr.hideleg);
            Arch::write_csr(Csr::Hvip, self.csr.hvip);
            Arch::write_csr(Csr::Hip, self.csr.hip);
            Arch::write_csr(Csr::Hie, self.csr.hie);
            Arch::write_csr(Csr::Hgeip, self.csr.hgeip);
            Arch::write_csr(Csr::Hgeie, self.csr.hgeie);
            Arch::write_csr(Csr::Henvcfg, self.csr.henvcfg);
            Arch::write_csr(Csr::Hcounteren, self.csr.hcounteren);
            Arch::write_csr(Csr::Htval, self.csr.htval);
            Arch::write_csr(Csr::Htinst, self.csr.htinst);
            Arch::write_csr(Csr::Hgatp, self.csr.hgatp);

            Arch::write_csr(Csr::Vsstatus, self.csr.vsstatus);
            Arch::write_csr(Csr::Vsie, self.csr.vsie);
            Arch::write_csr(Csr::Vstvec, self.csr.vstvec);
            Arch::write_csr(Csr::Vsscratch, self.csr.vsscratch);
            Arch::write_csr(Csr::Vsepc, self.csr.vsepc);
            Arch::write_csr(Csr::Vscause, self.csr.vscause);
            Arch::write_csr(Csr::Vstval, self.csr.vstval);
            Arch::write_csr(Csr::Vsip, self.csr.vsip);
            Arch::write_csr(Csr::Vsatp, self.csr.vsatp);
        }

        // Load virtual PMP registers into Miralis's own registers
        mctx.pmp.load_with_offset(
            &self.csr.pmpaddr,
            &self.csr.pmpcfg,
            mctx.pmp.virt_pmp_offset,
            self.nb_pmp,
        );
        // Deny all addresses by default if at least one PMP is implemented
        if self.nb_pmp > 0 {
            let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
            mctx.pmp
                .set_napot(last_pmp_idx, 0, usize::MAX, NO_PERMISSIONS);
        }
    }

    /// Loads the S-mode CSR registers into the virtual context and install sensible values (mostly
    /// 0) for running the virtual firmware in U-mode.
    pub unsafe fn switch_from_payload_to_firmware(&mut self, mctx: &mut MiralisContext) {
        // Now save M-mode registers which are (partially) exposed as S-mode registers.
        // For mstatus we read the current value and clear the two MPP bits to jump into U-mode
        // (virtual firmware) during the next mret.

        self.csr.mstatus = self.csr.mstatus & !mstatus::SSTATUS_FILTER
            | Arch::read_csr(Csr::Mstatus) & mstatus::SSTATUS_FILTER;
        Arch::set_mpp(Mode::U);
        Arch::write_csr(Csr::Mideleg, 0); // Do not delegate any interrupts
        Arch::write_csr(Csr::Medeleg, 0); // Do not delegate any exceptions

        self.csr.mie = Arch::read_csr(Csr::Mie);

        // Real mip.SEIE bit should not be different from virtual mip.SEIE as it is read-only in S-Mode or U-Mode.
        // But csrr is modified for SEIE and return the logical-OR of SEIE and the interrupt signal from interrupt
        // controller. (refer to documentation for further detail).
        // MSIE and MEIE could not be set by payload to it should be 0. The real value is read from hardware when
        // the firmware want to read virtual mip.
        let mip_hw_bits =
            Arch::read_csr(Csr::Mip) & !(mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        let mip_sw_bits = self.csr.mip & (mie::SEIE_FILTER | mie::MIDELEG_READ_ONLY_ZERO);
        self.csr.mip = mip_hw_bits | mip_sw_bits;

        let delegate_perf_counter_mask: usize = if DELEGATE_PERF_COUNTER { 1 } else { 0 };

        self.csr.mcounteren = Arch::write_csr(Csr::Mcounteren, delegate_perf_counter_mask);

        if mctx.hw.available_reg.senvcfg {
            self.csr.senvcfg = Arch::write_csr(Csr::Senvcfg, 0);
        }

        if mctx.hw.available_reg.menvcfg {
            self.csr.menvcfg = Arch::write_csr(Csr::Menvcfg, 0);
        }

        // If S extension is present - save the registers
        if mctx.hw.extensions.has_s_extension {
            self.csr.stvec = Arch::write_csr(Csr::Stvec, 0);
            self.csr.scounteren = Arch::write_csr(Csr::Scounteren, delegate_perf_counter_mask);
            self.csr.satp = Arch::write_csr(Csr::Satp, 0);

            self.csr.sscratch = Arch::write_csr(Csr::Sscratch, 0);
            self.csr.sepc = Arch::write_csr(Csr::Sepc, 0);
            self.csr.scause = Arch::write_csr(Csr::Scause, 0);

            self.csr.stval = Arch::write_csr(Csr::Stval, 0);
        }

        // If H extension is present - save the registers
        if mctx.hw.extensions.has_h_extension {
            self.csr.hstatus = Arch::read_csr(Csr::Hstatus);
            self.csr.hedeleg = Arch::read_csr(Csr::Hedeleg);
            self.csr.hideleg = Arch::read_csr(Csr::Hideleg);
            self.csr.hvip = Arch::read_csr(Csr::Hvip);
            self.csr.hip = Arch::read_csr(Csr::Hip);
            self.csr.hie = Arch::read_csr(Csr::Hie);
            self.csr.hgeip = Arch::read_csr(Csr::Hgeip); // Read only register, this write will have no effect
            self.csr.hgeie = Arch::read_csr(Csr::Hgeie);
            self.csr.henvcfg = Arch::read_csr(Csr::Henvcfg);
            self.csr.hcounteren = Arch::read_csr(Csr::Hcounteren);
            self.csr.htval = Arch::read_csr(Csr::Htval);
            self.csr.htinst = Arch::read_csr(Csr::Htinst);
            self.csr.hgatp = Arch::read_csr(Csr::Hgatp);

            self.csr.vsstatus = Arch::read_csr(Csr::Vsstatus);
            self.csr.vsie = Arch::read_csr(Csr::Vsie);
            self.csr.vstvec = Arch::read_csr(Csr::Vstvec);
            self.csr.vsscratch = Arch::read_csr(Csr::Vsscratch);
            self.csr.vsepc = Arch::read_csr(Csr::Vsepc);
            self.csr.vscause = Arch::read_csr(Csr::Vscause);
            self.csr.vstval = Arch::read_csr(Csr::Vstval);
            self.csr.vsip = Arch::read_csr(Csr::Vsip);
            self.csr.vsatp = Arch::read_csr(Csr::Vsatp);
        }

        // Remove Firmware PMP from the hardware
        mctx.pmp.clear_range(mctx.pmp.virt_pmp_offset, self.nb_pmp);
        // Allow all addresses by default
        let last_pmp_idx = mctx.pmp.nb_pmp as usize - 1;
        mctx.pmp.set_napot(last_pmp_idx, 0, usize::MAX, pmpcfg::RWX);
    }
}

// ———————————————————————— Register Setters/Getters ———————————————————————— //

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
            Csr::Mstatus => self.csr.mstatus & mstatus::MSTATUS_FILTER,
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
            Csr::Mvendorid => self.csr.mvendorid,
            Csr::Marchid => self.csr.marchid,
            Csr::Mimpid => self.csr.mimpid,
            Csr::Pmpcfg(pmp_cfg_idx) => {
                if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                }
                if pmp_cfg_idx >= self.nb_pmp / 8 {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmpcfg[pmp_cfg_idx / 2]
                    & VirtCsr::get_pmp_cfg_filter(pmp_cfg_idx, self.nb_pmp)
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= self.nb_pmp {
                    // This PMP is not emulated
                    return 0;
                }
                self.csr.pmpaddr[pmp_addr_idx]
            }
            Csr::Mcycle => self.csr.mcycle,
            Csr::Minstret => self.csr.minstret,
            Csr::Mhpmcounter(n) => self.csr.mhpmcounter[n],
            Csr::Mcountinhibit => self.csr.mcountinhibit,
            Csr::Mhpmevent(n) => self.csr.mhpmevent[n],
            Csr::Mcounteren => self.csr.mcounteren,
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
            Csr::Tselect => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION : read debug-mode specification
            Csr::Mepc => self.csr.mepc,
            Csr::Mcause => self.csr.mcause,
            Csr::Mtval => self.csr.mtval,
            //Supervisor-level CSRs
            Csr::Sstatus => self.get(Csr::Mstatus) & mstatus::SSTATUS_FILTER,
            Csr::Sie => self.get(Csr::Mie) & mie::SIE_FILTER,
            Csr::Stvec => self.csr.stvec,
            Csr::Scounteren => self.csr.scounteren,
            Csr::Senvcfg => self.csr.senvcfg,
            Csr::Sscratch => self.csr.sscratch,
            Csr::Sepc => self.csr.sepc,
            Csr::Scause => self.csr.scause,
            Csr::Stval => self.csr.stval,
            Csr::Sip => self.get(Csr::Mip) & mie::SIE_FILTER,
            Csr::Satp => self.csr.satp,
            Csr::Scontext => self.csr.scontext,
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
            // Unknown
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
        }
    }
}

impl HwRegisterContextSetter<Csr> for VirtContext {
    fn set_csr(&mut self, register: Csr, value: usize, mctx: &mut MiralisContext) {
        let hw = &mctx.hw;
        match register {
            Csr::Mhartid => (), // Read-only
            Csr::Mstatus => {
                // TODO: create some constant values
                let mut new_value = value & mstatus::MSTATUS_FILTER; //self.csr.mstatus;
                                                                     // MPP : 11 : write legal : 0,1,3
                let mpp = (value & mstatus::MPP_FILTER) >> mstatus::MPP_OFFSET;
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::MPP_OFFSET,
                    mstatus::MPP_FILTER,
                    if mpp == 0 || mpp == 1 || mpp == 3 {
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

                let pmp = &mut mctx.pmp;

                // When vMPRV transitions from 0 to 1, set up a PMP entry to protect all memory.
                // This allows catching accesses that occur with vMPRV=1, which require a special virtual access handler.
                // When vMPRV transitions back to 0, remove the protection.
                // pMPRV is never set to 1 outside of a virtual access handler.
                if mprv != previous_mprv {
                    log::trace!("vMPRV set to {:b}", mprv);
                    if mprv != 0 {
                        pmp.set_tor(0, usize::MAX, pmpcfg::X);
                    } else {
                        pmp.set_inactive(0, usize::MAX);
                    }
                    unsafe { Arch::sfencevma(None, None) };
                }

                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::MPRV_OFFSET,
                    mstatus::MPRV_FILTER,
                    mprv,
                );
                // MBE - We currently don't implement the feature as it is a very nice feature
                if new_value & MBE_FILTER != 0 {
                    todo!("MBE filter is not implemented - please implement it");
                }
                // SBE - We currently don't implement the feature as it is a very nice feature
                if new_value & SBE_FILTER != 0 {
                    todo!("SBE filter is not implemented - please implement it");
                }
                // UBE - We currently don't implement the feature as it is a very nice feature
                if new_value & UBE_FILTER != 0 {
                    todo!("UBE filter is not implemented - please implement it");
                }
                // TVM & TSR are read only when no S-mode is available
                if !mctx.hw.extensions.has_s_extension {
                    // TVM : 20
                    if !mctx.hw.extensions.has_s_extension {
                        VirtCsr::set_csr_field(
                            &mut new_value,
                            mstatus::TVM_OFFSET,
                            mstatus::TVM_FILTER,
                            0,
                        );
                    }
                    // TSR : 22
                    if !mctx.hw.extensions.has_s_extension {
                        VirtCsr::set_csr_field(
                            &mut new_value,
                            mstatus::TSR_OFFSET,
                            mstatus::TSR_FILTER,
                            0,
                        );
                    }
                }
                // FS : 13 : read-only 0 (NO S-MODE, F extension)
                if !mctx.hw.extensions.has_s_extension {
                    VirtCsr::set_csr_field(
                        &mut new_value,
                        mstatus::FS_OFFSET,
                        mstatus::FS_FILTER,
                        0,
                    );
                }
                // VS : 9 : read-only 0 (v registers)
                VirtCsr::set_csr_field(&mut new_value, mstatus::VS_OFFSET, mstatus::VS_FILTER, 0);
                // XS : 15 : read-only 0 (NO FS nor VS)
                VirtCsr::set_csr_field(&mut new_value, mstatus::XS_OFFSET, mstatus::XS_FILTER, 0);
                // SD : 63 : read-only 0 (if NO FS/VS/XS)
                VirtCsr::set_csr_field(
                    &mut new_value,
                    mstatus::SD_OFFSET,
                    mstatus::SD_FILTER,
                    if mctx.hw.extensions.has_s_extension {
                        let fs: usize = (value & mstatus::FS_FILTER) >> mstatus::FS_OFFSET;
                        if fs != 0 {
                            0b1
                        } else {
                            0b0
                        }
                    } else {
                        0
                    },
                );

                self.csr.mstatus = new_value;
            }
            Csr::Misa => {
                // misa shows the extensions available : we cannot have more than possible in hardware
                let arch_misa: usize = Arch::read_csr(Csr::Misa);
                // Update misa to a legal value
                self.csr.misa =
                    (value & arch_misa & misa::MISA_CHANGE_FILTER & !misa::DISABLED) | misa::MXL;

                if (self.csr.misa & misa::S) == 0 && mctx.hw.extensions.has_s_extension {
                    panic!("Miralis doesn't support deactivating the S mode extension, please implement the feature")
                }

                if (self.csr.misa & misa::H) == 0 && mctx.hw.extensions.has_h_extension {
                    panic!("Miralis doesn't support deactivating the H mode extension, please implement the feature")
                }
            }
            Csr::Mie => self.csr.mie = value & hw.interrupts & mie::MIE_WRITE_FILTER,
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
                self.csr.mip = value | (self.csr.mip & mie::MIDELEG_READ_ONLY_ZERO);
            }
            Csr::Mtvec => self.csr.mtvec = value,
            Csr::Mscratch => self.csr.mscratch = value,
            Csr::Mvendorid => (), // Read-only
            Csr::Marchid => (),   // Read-only
            Csr::Mimpid => (),    // Read-only
            Csr::Pmpcfg(pmp_cfg_idx) => {
                let mut value = value;
                if Csr::PMP_CFG_LOCK_MASK & value != 0 {
                    debug::warn_once!("PMP lock bits are not yet supported");
                    value &= !Csr::PMP_CFG_LOCK_MASK;
                }
                if pmp_cfg_idx % 2 == 1 {
                    // Illegal because we are in a RISCV64 setting
                    panic!("Illegal PMP_CFG {:?}", register)
                } else if pmp_cfg_idx >= self.nb_pmp / 8 {
                    // This PMP is not emulated, ignore changes
                    return;
                }
                self.csr.pmpcfg[pmp_cfg_idx / 2] = Csr::PMP_CFG_LEGAL_MASK
                    & value
                    & VirtCsr::get_pmp_cfg_filter(pmp_cfg_idx, self.nb_pmp);
            }
            Csr::Pmpaddr(pmp_addr_idx) => {
                if pmp_addr_idx >= mctx.hw.available_reg.nb_pmp {
                    // This PMP is not emulated, ignore
                    return;
                }
                self.csr.pmpaddr[pmp_addr_idx] = Csr::PMP_ADDR_LEGAL_MASK & value;
            }
            Csr::Mcycle => (),                                      // Read-only 0
            Csr::Minstret => (),                                    // Read-only 0
            Csr::Mhpmcounter(_counter_idx) => (),                   // Read-only 0
            Csr::Mcountinhibit => (),                               // Read-only 0
            Csr::Mhpmevent(_event_idx) => (),                       // Read-only 0
            Csr::Mcounteren => self.csr.mcounteren = value & 0b111, // Only show IR, TM and CY (for cycle, time and instret counters)
            Csr::Menvcfg => self.csr.menvcfg = value,
            Csr::Mseccfg => self.csr.mseccfg = value,
            Csr::Mconfigptr => (),                    // Read-only
            Csr::Medeleg => self.csr.medeleg = value, //TODO : some values need to be read-only 0
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
            Csr::Tselect => todo!(), // Read-only 0 when no triggers are implemented
            Csr::Tdata1 => todo!(),  // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata2 => todo!(),  // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Tdata3 => todo!(),  // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mcontext => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dcsr => todo!(),    // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dpc => todo!(),     // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch0 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Dscratch1 => todo!(), // TODO : NO INFORMATION IN THE SPECIFICATION
            Csr::Mepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                self.csr.mepc = value
            }
            Csr::Mcause => {
                let cause = MCause::new(value);
                match cause {
                    // Can only contain supported exception codes
                    MCause::UnknownException => (),
                    _ => self.csr.mcause = value,
                }
            }
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
                // Clear S bits
                let mie = self.get(Csr::Mie) & !mie::SIE_FILTER;
                // Set S bits to new value
                self.set_csr(Csr::Mie, mie | (value & mie::SIE_FILTER), mctx);
            }
            Csr::Stvec => self.csr.stvec = value,
            Csr::Scounteren => (), // Read-only 0
            Csr::Senvcfg => self.csr.senvcfg = value,
            Csr::Sscratch => self.csr.sscratch = value,
            Csr::Sepc => {
                if value > Plat::get_max_valid_address() {
                    return;
                }
                self.csr.sepc = value
            }
            Csr::Scause => {
                let cause = MCause::new(value);
                if cause.is_interrupt() {
                    // TODO : does not support interrupts
                    return;
                }
                match cause {
                    // Can only contain supported exception codes
                    MCause::UnknownException => (),
                    _ => self.csr.scause = value,
                }
            }
            Csr::Stval => self.csr.stval = value,
            Csr::Sip => {
                // Clear S bits
                let mip = self.get(Csr::Mip) & !mie::SIE_FILTER;
                // Set S bits to new value
                self.set_csr(Csr::Mip, mip | (value & mie::SIE_FILTER), mctx);
            }
            Csr::Satp => {
                self.csr.satp = value & satp::SATP_CHANGE_FILTER;
            }
            Csr::Scontext => todo!("No information in the specification"),
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
            // Unknown
            Csr::Unknown => panic!("Tried to access unknown CSR: {:?}", register),
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

/// Return the ID of the next interrupt to be delivered, if any.
fn get_next_interrupt(mie: usize, mip: usize, mideleg: usize) -> Option<usize> {
    let ints = mie & mip & !mideleg;
    if ints == 0 {
        None
    } else {
        // TODO: use the same priority as hardware.
        // Currently we serve the less significant bit first.
        Some(ints.trailing_zeros() as usize)
    }
}

// ————————————————————————————————— Tests —————————————————————————————————— //

#[cfg(test)]
mod tests {
    use core::usize;

    use super::get_next_interrupt;
    use crate::arch::{mie, mstatus, Arch, Architecture, Csr, Mode};
    use crate::host::MiralisContext;
    use crate::virt::VirtContext;
    use crate::HwRegisterContextSetter;

    /// We test value of mstatus.MPP.
    /// When switching from firmware to payload,
    /// virtual mstatus.MPP must to be S (because we are jumping to payload)
    /// and mstatus.MPP must be M (coming from Miralis).
    ///
    /// When switching from payload to firmware,
    /// virtual mstatus.MPP must to be S (coming from payload)
    /// and mstatus.MPP must be U (going to firmware).
    #[test]
    fn switch_context_mpp() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        ctx.csr.mstatus |= Mode::S.to_bits() << mstatus::MPP_OFFSET;

        unsafe { ctx.switch_from_firmware_to_payload(&mut mctx) }

        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP must be set to S mode (going to payload)"
        );

        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::M.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP must be set to M mode (coming from Miralis)"
        );

        // Simulate a trap
        unsafe { Arch::write_csr(Csr::Mstatus, Mode::S.to_bits() << mstatus::MPP_OFFSET) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        // VirtContext Mstatus.MPP has been set to M mode
        assert_eq!(
            ctx.csr.mstatus & mstatus::MPP_FILTER,
            Mode::S.to_bits() << mstatus::MPP_OFFSET,
            "VirtContext Mstatus.MPP has been set to S mode (coming from payload)"
        );

        // Mstatus.MPP has been set to U mode
        assert_eq!(
            Arch::read_csr(Csr::Mstatus) & mstatus::MPP_FILTER,
            Mode::U.to_bits() << mstatus::MPP_OFFSET,
            "Mstatus.MPP has been set to U mode (going to firmware)"
        );
    }

    /// We test value of mideleg when switching from payload to firmware.
    /// Mideleg must always be 0 when executing the firware.
    #[test]
    fn switch_to_firmware_mideleg() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        unsafe { Arch::write_csr(Csr::Mideleg, usize::MAX) };

        unsafe { ctx.switch_from_payload_to_firmware(&mut mctx) }

        assert_eq!(Arch::read_csr(Csr::Mideleg), 0, "Mideleg must be 0");
    }

    /// If the firmware wants to read the `mip` register after cleaning `vmip.SEIP`,
    /// and we don't sync `vmip.SEIP` with `mip.SEIP`, it can't know if there is an interrupt
    /// signal from the interrupt controller as the CSR read will be a logical-OR of the
    /// signal and `mip.SEIP` (which is one), and so always 1.
    /// If vmip.SEIP is 0, CSR read of mip.SEIP should return the interrupt signal.
    ///
    /// Then, we need to synchronize vmip.SEIP with mip.SEIP.
    #[test]
    fn csrr_external_interrupt() {
        let hw = unsafe { Arch::detect_hardware() };
        let mut mctx = MiralisContext::new(hw);
        let mut ctx = VirtContext::new(0, mctx.hw.available_reg.nb_pmp, mctx.hw.extensions.clone());

        // This should set mip.SEIP
        ctx.set_csr(Csr::Mip, mie::SEIE_FILTER, &mut mctx);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            mie::SEIE_FILTER,
            "mip.SEIP must be 1"
        );

        // This should clear mip.SEIP
        ctx.set_csr(Csr::Mip, 0, &mut mctx);

        assert_eq!(
            Arch::read_csr(Csr::Mip) & mie::SEIE_FILTER,
            0,
            "mip.SEIP must be 0"
        );
    }

    #[test]
    fn next_interrupt() {
        assert_eq!(get_next_interrupt(0b000, 0b000, 0b000), None);
        assert_eq!(get_next_interrupt(0b010, 0b000, 0b000), None);
        assert_eq!(get_next_interrupt(0b000, 0b010, 0b000), None);
        assert_eq!(get_next_interrupt(0b010, 0b010, 0b010), None);

        assert_eq!(get_next_interrupt(0b001, 0b001, 0b000), Some(0));
        assert_eq!(get_next_interrupt(0b011, 0b011, 0b000), Some(0));
        assert_eq!(get_next_interrupt(0b010, 0b010, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b010, 0b011, 0b000), Some(1));
        assert_eq!(get_next_interrupt(0b011, 0b011, 0b001), Some(1));
    }
}
