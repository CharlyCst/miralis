// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::confidential_flow::handlers::sbi::{SbiRequest, SbiResponse};
use crate::ace::confidential_flow::{ApplyToConfidentialHart, ConfidentialFlow};
use crate::ace::core::architecture::riscv::sbi::CovgExtension;
use crate::ace::core::architecture::{GeneralPurposeRegister, SharedPage};
use crate::ace::core::control_data::{ConfidentialHart, ResumableOperation};
use crate::ace::core::memory_layout::ConfidentialVmPhysicalAddress;
use crate::ace::error::Error;
use crate::ace::non_confidential_flow::DeclassifyToHypervisor;
use crate::ensure;

/// Handles a request from the confidential VM about creating a shared page.
///
/// Control flows to the hypervisor when the sharing of the given `guest physical address` is allowed. The hypervisor is requested to
/// allocate a page of non-confidential memory and return back the `host physical address` of this page. Control flows back to the
/// confidential hart if the request was invalid, e.g., the `guest physical address` was not correct.
pub struct SharePageRequest {
    pub address: ConfidentialVmPhysicalAddress,
    pub size: usize,
}

impl SharePageRequest {
    pub fn from_confidential_hart(confidential_hart: &ConfidentialHart) -> Self {
        Self {
            address: ConfidentialVmPhysicalAddress::new(
                confidential_hart.gprs().read(GeneralPurposeRegister::a0),
            ),
            size: confidential_hart.gprs().read(GeneralPurposeRegister::a1),
        }
    }

    pub fn handle(self, confidential_flow: ConfidentialFlow) -> ! {
        match self.share_page_sbi_request() {
            Ok(sbi_request) => confidential_flow
                .set_resumable_operation(ResumableOperation::SharePage(self))
                .into_non_confidential_flow()
                .declassify_and_exit_to_hypervisor(DeclassifyToHypervisor::SbiRequest(sbi_request)),
            Err(error) => confidential_flow.apply_and_exit_to_confidential_hart(
                ApplyToConfidentialHart::SbiResponse(SbiResponse::error(error)),
            ),
        }
    }

    fn share_page_sbi_request(&self) -> Result<SbiRequest, Error> {
        ensure!(
            self.address.usize() % SharedPage::SIZE.in_bytes() == 0,
            Error::AddressNotAligned()
        )?;
        ensure!(
            self.size == SharedPage::SIZE.in_bytes(),
            Error::InvalidParameter()
        )?;
        Ok(SbiRequest::new(
            CovgExtension::EXTID,
            CovgExtension::SBI_EXT_COVG_SHARE_MEMORY,
            self.address.usize(),
            self.size,
        ))
    }
}
