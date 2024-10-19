// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::core::architecture::GeneralPurposeRegister;
use crate::ace::core::control_data::HypervisorHart;
use crate::ace::error::Error;
use crate::ace::non_confidential_flow::handlers::supervisor_binary_interface::SbiResponse;
use crate::ace::non_confidential_flow::{ApplyToHypervisorHart, NonConfidentialFlow};
use crate::debug;

pub struct InvalidCall {
    extension_id: usize,
    function_id: usize,
}

impl InvalidCall {
    pub fn from_hypervisor_hart(hypervisor_hart: &HypervisorHart) -> Self {
        Self {
            extension_id: hypervisor_hart.gprs().read(GeneralPurposeRegister::a7),
            function_id: hypervisor_hart.gprs().read(GeneralPurposeRegister::a6),
        }
    }

    pub fn handle(self, non_confidential_flow: NonConfidentialFlow) -> ! {
        debug!(
            "Not supported call {:x} {:x}",
            self.extension_id, self.function_id
        );
        let error = Error::InvalidCall(self.extension_id, self.function_id);
        non_confidential_flow.apply_and_exit_to_hypervisor(ApplyToHypervisorHart::SbiResponse(
            SbiResponse::error(error),
        ))
    }
}
