// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::confidential_flow::handlers::sbi::{SbiRequest, SbiResponse};
use crate::ace::confidential_flow::{ApplyToConfidentialHart, ConfidentialFlow};
use crate::ace::core::architecture::riscv::sbi::CovgExtension;
use crate::ace::core::architecture::GeneralPurposeRegister;
use crate::ace::core::control_data::{ConfidentialHart, ControlDataStorage, ResumableOperation};
use crate::ace::non_confidential_flow::DeclassifyToHypervisor;

pub struct AllowExternalInterrupt {
    interrupt_id: usize,
}

impl AllowExternalInterrupt {
    pub fn from_confidential_hart(confidential_hart: &ConfidentialHart) -> Self {
        Self {
            interrupt_id: confidential_hart.gprs().read(GeneralPurposeRegister::a0),
        }
    }

    pub fn handle(self, confidential_flow: ConfidentialFlow) -> ! {
        match ControlDataStorage::try_confidential_vm(
            confidential_flow.confidential_vm_id(),
            |mut confidential_vm| Ok(confidential_vm.allow_external_interrupt(self.interrupt_id)),
        ) {
            Ok(_) => confidential_flow
                .set_resumable_operation(ResumableOperation::SbiRequest())
                .into_non_confidential_flow()
                .declassify_and_exit_to_hypervisor(DeclassifyToHypervisor::SbiRequest(
                    self.sbi_covg_allow_ext_interrupt(),
                )),
            Err(error) => confidential_flow.apply_and_exit_to_confidential_hart(
                ApplyToConfidentialHart::SbiResponse(SbiResponse::error(error)),
            ),
        }
    }

    fn sbi_covg_allow_ext_interrupt(&self) -> SbiRequest {
        SbiRequest::new(
            CovgExtension::EXTID,
            CovgExtension::SBI_EXT_COVG_ALLOW_EXT_INTERRUPT,
            self.interrupt_id,
            0,
        )
    }
}
