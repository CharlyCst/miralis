// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::confidential_flow::ConfidentialFlow;
use crate::ace::core::architecture::riscv::sbi::BaseExtension::*;
use crate::ace::core::architecture::riscv::sbi::CovhExtension::*;
use crate::ace::core::architecture::riscv::sbi::NaclExtension::*;
use crate::ace::core::architecture::riscv::sbi::NaclSharedMemory;
use crate::ace::core::architecture::riscv::sbi::SbiExtension::*;
use crate::ace::core::architecture::TrapCause;
use crate::ace::core::architecture::TrapCause::*;
use crate::ace::core::control_data::{ConfidentialVmId, HardwareHart, HypervisorHart};
use crate::ace::error::Error;
use crate::ace::non_confidential_flow::handlers::cove_hypervisor_extension::{
    DestroyConfidentialVm, GetSecurityMonitorInfo, PromoteToConfidentialVm, RunConfidentialHart,
};
use crate::ace::non_confidential_flow::handlers::nested_acceleration_extension::{NaclProbeFeature, NaclSetupSharedMemory};
use crate::ace::non_confidential_flow::handlers::opensbi::{DelegateToOpensbi, ProbeSbiExtension};
use crate::ace::non_confidential_flow::handlers::supervisor_binary_interface::InvalidCall;
use crate::ace::non_confidential_flow::{ApplyToHypervisorHart, DeclassifyToHypervisor};
use crate::ace_to_miralis_ctx_switch;

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
    pub(crate) const CTX_SWITCH_ERROR_MSG: &'static str = "Bug: invalid argument provided by the assembly context switch";

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
        //log::info!("Back in the handler");
        // Below unsafe is ok because the lightweight context switch (assembly) guarantees that it provides us with a valid pointer to the
        // hardware hart's dump area in main memory. This area in main memory is exclusively owned by the physical hart executing this code.
        // Specifically, every physical hart has its own are in the main memory and its `mscratch` register stores the address. See the
        // `initialization` procedure for more details.
        let flow = unsafe { Self::create(hart_ptr.as_mut().expect(Self::CTX_SWITCH_ERROR_MSG)) };


        /*log::error!("We reached this code, so it is a good sign, now the next step is to implement this part");
        log::warn!("Mcause : 0x{:x}", flow.hardware_hart.hypervisor_hart.hypervisor_hart_state.csrs.mcause.read());
        log::warn!("Mepc : 0x{:x}", flow.hardware_hart.hypervisor_hart.hypervisor_hart_state.csrs.mepc.read());*/

        // End Modification for Miralis
        match TrapCause::from_hart_architectural_state(flow.hypervisor_hart().hypervisor_hart_state()) {
            Interrupt => todo!("Implement interrupt"), // DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            IllegalInstruction => todo!("Implement illegal instruction"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            LoadAddressMisaligned => todo!("Implement load address misaligned"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            LoadAccessFault => todo!("Load access fault"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            StoreAddressMisaligned => todo!("Store Address Misaligned"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            StoreAccessFault => todo!("Implement Store AccessFault"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            // HsEcall(Base(ProbeExtension)) => ProbeSbiExtension::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Covh(TsmGetInfo)) => todo!("Enable tsm get info"), //GetSecurityMonitorInfo::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Covh(PromoteToTvm)) => todo!("Enable promote to vm"), //PromoteToConfidentialVm::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Covh(TvmVcpuRun)) => todo!("Enable tvm to cpu run"), //RunConfidentialHart::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Covh(DestroyTvm)) => todo!("Enable destroy tvm"), //DestroyConfidentialVm::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Covh(_)) => todo!("Enable Covh"), //InvalidCall::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Nacl(ProbeFeature)) => todo!("Enable probe feature"), //NaclProbeFeature::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Nacl(SetupSharedMemory)) => todo!("Enable setup shared memory"), //NaclSetupSharedMemory::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            HsEcall(Nacl(_)) => todo!("Enable nacl"), //InvalidCall::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            // TODO: Add handling of the other case
            HsEcall(_) => ace_to_miralis_ctx_switch(flow.hardware_hart), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            MachineEcall => panic!("Machine ecall, is it normal (it might be)"), //DelegateToOpensbi::from_hypervisor_hart(flow.hypervisor_hart()).handle(flow),
            trap_reason => panic!("Bug: Incorrect interrupt delegation configuration: {:?}", trap_reason),
        }
    }

    /// Tries to traverse to confidential flow of the finite state machine (FSM). Returns error if the identifier of a confidential VM or
    /// hart are incorrect or cannot be scheduled for execution.
    pub fn into_confidential_flow(
        self, confidential_vm_id: ConfidentialVmId, confidential_hart_id: usize,
    ) -> Result<(usize, ConfidentialFlow<'a>), (NonConfidentialFlow<'a>, Error)> {
        ConfidentialFlow::enter_from_non_confidential_flow(self.hardware_hart, confidential_vm_id, confidential_hart_id)
            .map_err(|(hardware_hart, error)| (Self::create(hardware_hart), error))
    }

    pub fn declassify_to_hypervisor_hart(mut self, declassify: DeclassifyToHypervisor) -> Self {
        match declassify {
            DeclassifyToHypervisor::SbiRequest(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
            DeclassifyToHypervisor::SbiResponse(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
            DeclassifyToHypervisor::Interrupt(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
            DeclassifyToHypervisor::MmioLoadRequest(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
            DeclassifyToHypervisor::MmioStoreRequest(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
            DeclassifyToHypervisor::EnabledInterrupts(v) => v.declassify_to_hypervisor_hart(self.hypervisor_hart_mut()),
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
    pub(crate) fn apply_and_exit_to_hypervisor(mut self, transformation: ApplyToHypervisorHart) -> ! {
        match transformation {
            ApplyToHypervisorHart::SbiResponse(v) => v.apply_to_hypervisor_hart(self.hypervisor_hart_mut()),
            //ApplyToHypervisorHart::OpenSbiResponse(v) => v.apply_to_hypervisor_hart(self.hypervisor_hart_mut()),
            ApplyToHypervisorHart::SetSharedMemory(v) => v.apply_to_hypervisor_hart(self.hypervisor_hart_mut()),
        }
        unsafe { exit_to_hypervisor_asm() }
    }

    /// Swaps the mscratch register value with the original mascratch value used by OpenSBI. This function must be
    /// called before executing any OpenSBI function. We can remove this once we get rid of the OpenSBI firmware.
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
