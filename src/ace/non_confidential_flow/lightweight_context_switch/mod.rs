// SPDX-FileCopyrightText: 2023 IBM Corporation
// SPDX-FileContributor: Wojciech Ozga <woz@zurich.ibm.com>, IBM Research - Zurich
// SPDX-License-Identifier: Apache-2.0

core::arch::global_asm!(
    core::include_str!("enter_from_hypervisor_or_vm.S"),
    core::include_str!("exit_to_hypervisor.S"),
    // below is a boilerplate code to glue Rust and Assembly code.
    HART_RA_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_RA_OFFSET,
    HART_SP_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_SP_OFFSET,
    HART_GP_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_GP_OFFSET,
    HART_TP_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_TP_OFFSET,
    HART_T0_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T0_OFFSET,
    HART_T1_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T1_OFFSET,
    HART_T2_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T2_OFFSET,
    HART_S0_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S0_OFFSET,
    HART_S1_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S1_OFFSET,
    HART_A0_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A0_OFFSET,
    HART_A1_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A1_OFFSET,
    HART_A2_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A2_OFFSET,
    HART_A3_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A3_OFFSET,
    HART_A4_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A4_OFFSET,
    HART_A5_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A5_OFFSET,
    HART_A6_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A6_OFFSET,
    HART_A7_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_A7_OFFSET,
    HART_S2_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S2_OFFSET,
    HART_S3_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S3_OFFSET,
    HART_S4_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S4_OFFSET,
    HART_S5_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S5_OFFSET,
    HART_S6_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S6_OFFSET,
    HART_S7_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S7_OFFSET,
    HART_S8_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S8_OFFSET,
    HART_S9_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S9_OFFSET,
    HART_S10_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S10_OFFSET,
    HART_S11_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_S11_OFFSET,
    HART_T3_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T3_OFFSET,
    HART_T4_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T4_OFFSET,
    HART_T5_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T5_OFFSET,
    HART_T6_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_T6_OFFSET,
    HART_MEPC_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_MEPC_OFFSET,
    HART_MSTATUS_OFFSET = const crate::ace::core::architecture::riscv::hart_architectural_state::HART_MSTATUS_OFFSET,
    HART_STACK_ADDRESS_OFFSET = const crate::ace::core::control_data::HART_STACK_ADDRESS_OFFSET,
);