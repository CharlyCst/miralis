// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0
use crate::ace::core::architecture::riscv::sbi::SBI_SUCCESS;
use crate::ace::core::architecture::riscv::specification::ECALL_INSTRUCTION_LENGTH;
use crate::ace::core::architecture::GeneralPurposeRegister;
use crate::ace::core::control_data::HypervisorHart;
use crate::ace::error::Error;

pub struct SbiResponse {
    a0: usize,
    a1: usize,
}

impl SbiResponse {
    pub fn success() -> Self {
        Self::success_with_code(0)
    }

    pub fn success_with_code(code: usize) -> Self {
        Self {
            a0: SBI_SUCCESS as usize,
            a1: code,
        }
    }

    pub fn error(error: Error) -> Self {
        Self {
            a0: error.sbi_error_code(),
            a1: 0,
        }
    }

    pub fn apply_to_hypervisor_hart(&self, hypervisor_hart: &mut HypervisorHart) {
        hypervisor_hart
            .csrs_mut()
            .mepc
            .add(ECALL_INSTRUCTION_LENGTH);
        hypervisor_hart
            .gprs_mut()
            .write(GeneralPurposeRegister::a0, self.a0);
        hypervisor_hart
            .gprs_mut()
            .write(GeneralPurposeRegister::a1, self.a1);
    }
}
