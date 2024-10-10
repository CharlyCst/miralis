// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::confidential_flow::ConfidentialFlow;
use crate::ace::core::architecture::riscv::sbi::BaseExtension::*;
use crate::ace::core::architecture::riscv::sbi::CovhExtension::*;
use crate::ace::core::architecture::riscv::sbi::NaclExtension::*;
use crate::ace::core::architecture::riscv::sbi::NaclSharedMemory;
use crate::ace::core::architecture::riscv::sbi::SbiExtension::*;
use crate::ace::core::architecture::sbi::{CovhExtension, NaclExtension};
use crate::ace::core::architecture::TrapCause;
use crate::ace::core::architecture::TrapCause::*;
use crate::ace::core::control_data::{ConfidentialVmId, HardwareHart, HypervisorHart};
use crate::ace::error::Error;
use crate::ace::non_confidential_flow::handlers::cove_hypervisor_extension::{
    DestroyConfidentialVm, GetSecurityMonitorInfo, PromoteToConfidentialVm, RunConfidentialHart,
};
use crate::ace::non_confidential_flow::handlers::nested_acceleration_extension::{
    NaclProbeFeature, NaclSetupSharedMemory,
};
use crate::ace::non_confidential_flow::handlers::opensbi::ProbeSbiExtension;
use crate::ace::non_confidential_flow::handlers::supervisor_binary_interface::{
    InvalidCall, SbiResponse,
};
use crate::ace::non_confidential_flow::{ApplyToHypervisorHart, DeclassifyToHypervisor};
use crate::policy::ace::ace_to_miralis_ctx_switch;

extern "C" {
    /// To ensure safety, specify all possible valid states that KVM expects to see and prove that security monitor
    /// never returns to KVM with other state. For example, only a subset of exceptions/interrupts can be handled by KVM.
    /// KVM kill the vcpu if it receives unexpected exception because it does not know what to do with it.
    fn exit_to_hypervisor_asm() -> !;
}

/// Represents the non-confidential part of the finite state machine (FSM), implementing router and exit nodes. It encapsulates the
/// HardwareHart instance, which is never exposed. It invokes handlers providing them temporary read access to hypervisor hart state.
pub struct NonConfidentialFlow<'a> {
    hardware_hart: &'a mut HardwareHart,
}

impl<'a> NonConfidentialFlow<'a> {
    pub(crate) const CTX_SWITCH_ERROR_MSG: &'static str =
        "Bug: invalid argument provided by the assembly context switch";

    /// Creates an instance of the `NonConfidentialFlow`. A confidential hart must not be assigned to the hardware hart.
    pub fn create(hardware_hart: &'a mut HardwareHart) -> Self {
        assert!(hardware_hart.confidential_hart().is_dummy());
        Self { hardware_hart }
    }

    /// Routes control flow execution based on the trap cause. This is an entry node (Assembly->Rust) of the non-confidential flow part of
    /// the finite state machine (FSM).
    ///
    /// # Safety
    ///
    /// * A confidential hart must not be assigned to the hardware hart.
    /// * This function must only be invoked by the assembly lightweight context switch.
    /// * Pointer is a not null and points to a memory region owned by the physical hart executing this code.
    #[no_mangle]
    unsafe extern "C" fn route_trap_from_hypervisor_or_vm(hart_ptr: *mut HardwareHart) -> ! {
        // Below unsafe is ok because the lightweight context switch (assembly) guarantees that it provides us with a valid pointer to the
        // hardware hart's dump area in main memory. This area in main memory is exclusively owned by the physical hart executing this code.
        // Specifically, every physical hart has its own are in the main memory and its `mscratch` register stores the address. See the
        // `initialization` procedure for more details.
        let flow = unsafe { Self::create(hart_ptr.as_mut().expect(Self::CTX_SWITCH_ERROR_MSG)) };

        let current_cause = TrapCause::from_hart_architectural_state(
            flow.hypervisor_hart().hypervisor_hart_state(),
        );

        // End Modification for Miralis
        match current_cause {
            Interrupt => ace_to_miralis_ctx_switch(flow.hardware_hart), // DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            IllegalInstruction => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            LoadAddressMisaligned => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            LoadAccessFault => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            StoreAddressMisaligned => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            StoreAccessFault => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Base(ProbeExtension)) => {
                let extension = ProbeSbiExtension::from_hypervisor_hart(flow.hypervisor_hart());
                if extension.extension_id == CovhExtension::EXTID
                    || extension.extension_id == NaclExtension::EXTID
                {
                    flow.apply_and_exit_to_hypervisor(ApplyToHypervisorHart::SbiResponse(
                        SbiResponse::success_with_code(1),
                    ))
                } else {
                    ace_to_miralis_ctx_switch(flow.hardware_hart)
                }
            }
            HsEcall(Covh(TsmGetInfo)) => {
                GetSecurityMonitorInfo::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Covh(PromoteToTvm)) => {
                PromoteToConfidentialVm::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Covh(TvmVcpuRun)) => {
                RunConfidentialHart::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Covh(DestroyTvm)) => {
                DestroyConfidentialVm::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Covh(_)) => {
                InvalidCall::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Nacl(ProbeFeature)) => {
                NaclProbeFeature::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Nacl(SetupSharedMemory)) => {
                NaclSetupSharedMemory::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            HsEcall(Nacl(_)) => {
                InvalidCall::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow)
            }
            // TODO: Add handling of the other case
            HsEcall(_) => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            MachineEcall => panic!("Machine ecall, is it normal (it might be)"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            trap_reason => {
                let mepc = flow.hardware_hart.hypervisor_hart.hypervisor_hart_state.csrs.mepc.read();
                log::error!("Error : 0x{:x}", mepc);

                panic!(
                    "Bug: Incorrect interrupt delegation configuration: {:?}",
                    trap_reason
                );
            }
        }
    }

    /// Tries to traverse to confidential flow of the finite state machine (FSM). Returns error if the identifier of a confidential VM or
    /// hart are incorrect or cannot be scheduled for execution.
    pub fn into_confidential_flow(
        self,
        confidential_vm_id: ConfidentialVmId,
        confidential_hart_id: usize,
    ) -> Result<(usize, ConfidentialFlow<'a>), (NonConfidentialFlow<'a>, Error)> {
        ConfidentialFlow::enter_from_non_confidential_flow(
            self.hardware_hart,
            confidential_vm_id,
            confidential_hart_id,
        )
        .map_err(|(hardware_hart, error)| (Self::create(hardware_hart), error))
    }

    pub fn declassify_to_hypervisor_hart(mut self, declassify: DeclassifyToHypervisor) -> Self {
        match declassify {
            DeclassifyToHypervisor::SbiRequest(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            DeclassifyToHypervisor::SbiResponse(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            DeclassifyToHypervisor::Interrupt(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            DeclassifyToHypervisor::MmioLoadRequest(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            DeclassifyToHypervisor::MmioStoreRequest(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            DeclassifyToHypervisor::EnabledInterrupts(v) => {
                v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
        }
        self
    }

    /// Resumes execution of the hypervisor hart and declassifies information from a confidential VM to the hypervisor. This is an exit node
    /// (Rust->Assembly) of the non-confidential part of the finite state machine (FSM), executed as a result of confidential VM
    /// execution (there was context switch between security domains).
    pub fn declassify_and_exit_to_hypervisor(self, declassify: DeclassifyToHypervisor) -> ! {
        self.declassify_to_hypervisor_hart(declassify);
        unsafe { exit_to_hypervisor_asm() }
    }

    /// Resumes execution of the hypervisor hart and applies state transformation. This is an exit node (Rust->Assembly) of the
    /// non-confidential part of the finite state machine (FSM), executed as a result of processing hypervisor request (there was no
    /// context switch between security domains).
    pub(crate) fn apply_and_exit_to_hypervisor(
        mut self,
        transformation: ApplyToHypervisorHart,
    ) -> ! {
        match transformation {
            ApplyToHypervisorHart::SbiResponse(v) => {
                v.apply_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
            //ApplyToHypervisorHart::OpenSbiResponse(v) => v.apply_to_hypervisor_hart(self.hypervisor_hart_mut()),
            ApplyToHypervisorHart::SetSharedMemory(v) => {
                v.apply_to_hypervisor_hart(self.hypervisor_hart_mut())
            }
        }
        unsafe { exit_to_hypervisor_asm() }
    }

    /// Swaps the mscratch register value with the original mascratch value used by OpenSBI. This function must be
    /// called before executing any OpenSBI function. We can remove this once we get rid of the OpenSBI firmware.
    #[allow(dead_code)]
    pub fn swap_mscratch(&mut self) {
        self.hardware_hart.swap_mscratch()
    }

    pub fn shared_memory(&self) -> &NaclSharedMemory {
        self.hypervisor_hart().shared_memory()
    }

    fn hypervisor_hart_mut(&mut self) -> &mut HypervisorHart {
        self.hardware_hart.hypervisor_hart_mut()
    }

    fn hypervisor_hart(&self) -> &HypervisorHart {
        &self.hardware_hart.hypervisor_hart()
    }
}



use core::arch::asm;

use crate::arch::pmp::pmpcfg;

/// Macro to read the `pmpcfgx` register where `x` is an argument (0-3).
/// Returns the value of the specified `pmpcfgx` register as a 64-bit unsigned integer.
fn read_pmpcfg(idx: usize) -> usize {
    let value: usize;
    unsafe {
        match idx {
            0 => asm!("csrr {}, pmpcfg0", out(reg) value),
            1 => asm!("csrr {}, pmpcfg0", out(reg) value),
            2 => asm!("csrr {}, pmpcfg0", out(reg) value),
            3 => asm!("csrr {}, pmpcfg0", out(reg) value),
            4 => asm!("csrr {}, pmpcfg0", out(reg) value),
            5 => asm!("csrr {}, pmpcfg0", out(reg) value),
            6 => asm!("csrr {}, pmpcfg0", out(reg) value),
            7 => asm!("csrr {}, pmpcfg0", out(reg) value),
            8 => asm!("csrr {}, pmpcfg2", out(reg) value),
            9 => asm!("csrr {}, pmpcfg2", out(reg) value),
            10 => asm!("csrr {}, pmpcfg2", out(reg) value),
            11 => asm!("csrr {}, pmpcfg2", out(reg) value),
            12 => asm!("csrr {}, pmpcfg2", out(reg) value),
            13 => asm!("csrr {}, pmpcfg2", out(reg) value),
            14 => asm!("csrr {}, pmpcfg2", out(reg) value),
            15 => asm!("csrr {}, pmpcfg2", out(reg) value),
            _ => panic!("Invalid pmpcfg index: {}", idx),
        }
    }
    value
}

pub fn read_pmpaddr(index: usize) -> u64 {
    if index >= 16 {
        panic!("Invalid PMP address register index: {}", index);
    }

    // The `pmpaddr` CSR indices start at 0x3B0 for `pmpaddr0`.
    let pmpaddr_base = 0x3B0;

    // Compute the CSR address for the specified `pmpaddr` register.
    let csr_address = pmpaddr_base + index;

    // Read the CSR value using inline assembly.
    let value: u64;
    unsafe {
        match index {
            0 => asm!("csrr {}, pmpaddr0", out(reg) value),
            1 => asm!("csrr {}, pmpaddr1", out(reg) value),
            2 => asm!("csrr {}, pmpaddr2", out(reg) value),
            3 => asm!("csrr {}, pmpaddr3", out(reg) value),
            4 => asm!("csrr {}, pmpaddr4", out(reg) value),
            5 => asm!("csrr {}, pmpaddr5", out(reg) value),
            6 => asm!("csrr {}, pmpaddr6", out(reg) value),
            7 => asm!("csrr {}, pmpaddr7", out(reg) value),
            8 => asm!("csrr {}, pmpaddr8", out(reg) value),
            9 => asm!("csrr {}, pmpaddr9", out(reg) value),
            10 => asm!("csrr {}, pmpaddr10", out(reg) value),
            11 => asm!("csrr {}, pmpaddr11", out(reg) value),
            12 => asm!("csrr {}, pmpaddr12", out(reg) value),
            13 => asm!("csrr {}, pmpaddr13", out(reg) value),
            14 => asm!("csrr {}, pmpaddr14", out(reg) value),
            15 => asm!("csrr {}, pmpaddr15", out(reg) value),
            _ => panic!("Invalid pmpcfg index: {}", index),
        }
    }

    value
}

pub fn get_configuration(index: usize) -> u8 {
    let reg_idx = index / 8;
    let inner_idx = index % 8;
    let reg = read_pmpcfg(reg_idx);
    let cfg = (reg >> (inner_idx * 8)) & 0xff;
    cfg as u8
}

pub(crate) fn print_pmp() {
    let mut prev_addr2: u64 = 0;
    for idx in 0..16 {
        let cfg = get_configuration(idx);
        let addr = read_pmpaddr(idx);
        let prev_addr = prev_addr2;

        prev_addr2 = addr;

        match cfg & pmpcfg::A_MASK {
            pmpcfg::NA4 => {
                let addr = addr << 2;
                log::info!("Implement A-mask ;");
            }
            pmpcfg::NAPOT => {
                let trailing_ones = addr.trailing_ones();
                let addr_mask = !((1 << trailing_ones) - 1);
                let addr = (addr & addr_mask) << 2;
                let shift = trailing_ones + 3;
                log::info!(
                    " From - To : 0x{:x} --> 0x{:x} and permissions : 0x{:x}",
                    addr,
                    (1 << shift),
                    cfg & pmpcfg::RWX
                );
            }
            pmpcfg::TOR => {
                // if prev_addr is bigger then that entry does not match anything
                if prev_addr >= addr {
                    continue;
                }
                let size = addr - prev_addr;
                log::info!(
                    " From - To : 0x{:x} --> 0x{:x} and permissions : 0x{:x}",
                    prev_addr,
                    size,
                    cfg & pmpcfg::RWX
                );
            }
            _ => {
                log::info!("Inactive pmp entry");
                // Inactive PMP entry
                continue;
            }
        }
    }
}
