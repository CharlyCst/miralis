// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::confidential_flow::handlers::sbi::{SbiRequest, SbiResponse};
use crate::ace::confidential_flow::{ApplyToConfidentialHart, ConfidentialFlow};
use crate::ace::core::architecture::sbi::CovgExtension;
use crate::ace::core::architecture::{GeneralPurposeRegister, PageSize};
use crate::ace::core::control_data::{ConfidentialHart, ConfidentialVmMmioRegion, ControlDataStorage, ResumableOperation};
use crate::ace::error::Error;
use crate::ace::non_confidential_flow::DeclassifyToHypervisor;
use crate::ensure;

pub struct RemoveMmioRegion {
    region_start_address: usize,
    region_length: usize,
}

impl RemoveMmioRegion {
    pub fn from_confidential_hart(confidential_hart: &ConfidentialHart) -> Self {
        Self {
            region_start_address: confidential_hart.gprs().read(GeneralPurposeRegister::a0),
            region_length: confidential_hart.gprs().read(GeneralPurposeRegister::a1),
        }
    }

    pub fn handle(self, confidential_flow: ConfidentialFlow) -> ! {
        match ControlDataStorage::try_confidential_vm(confidential_flow.confidential_vm_id(), |mut confidential_vm| {
            ensure!(self.region_start_address % PageSize::Size4KiB.in_bytes() == 0, Error::AddressNotAligned())?;
            ensure!(self.region_length % PageSize::Size4KiB.in_bytes() == 0, Error::AddressNotAligned())?;
            Ok(confidential_vm.remove_mmio_region(&ConfidentialVmMmioRegion::new(self.region_start_address, self.region_length)))
        }) {
            Ok(_) => confidential_flow
                .set_resumable_operation(ResumableOperation::SbiRequest())
                .into_non_confidential_flow()
                .declassify_and_exit_to_hypervisor(DeclassifyToHypervisor::SbiRequest(self.sbi_remove_mmio_region())),
            Err(error) => {
                confidential_flow.apply_and_exit_to_confidential_hart(ApplyToConfidentialHart::SbiResponse(SbiResponse::error(error)))
            }
        }
    }

    fn sbi_remove_mmio_region(&self) -> SbiRequest {
        SbiRequest::new(CovgExtension::EXTID, CovgExtension::SBI_EXT_COVG_REMOVE_MMIO_REGION, self.region_start_address, self.region_length)
    }
}