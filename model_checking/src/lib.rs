extern crate core;

use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::{mie, write_pmp, Register};
use miralis::decoder::Instr;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use sail_decoder::encdec_backwards;
use sail_model::{execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA, execute_SRET, execute_WFI, pmpCheck, readCSR, step_interrupts_only, writeCSR, AccessType, ExceptionType, Privilege, ast};
use sail_prelude::{sys_pmp_count, BitField, BitVector};

use crate::adapters::{
    ast_to_miralis_instr, decode_csr_register, pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis,
    sail_to_miralis,
};

#[macro_use]
mod symbolic;
mod adapters;
mod execute;

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn mret() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    ctx.emulate_mret(&mut mctx);

    execute_MRET(&mut sail_ctx);

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "mret instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn sret() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    ctx.emulate_sret(&mut mctx);

    execute_SRET(&mut sail_ctx);

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "sret instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn wfi() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    ctx.emulate_wfi(&mut mctx);

    execute_WFI(&mut sail_ctx);

    // This field is used only in Miralis. We set it to false otherwise the assertions fails.
    ctx.is_wfi = false;

    assert_eq!(
        ctx,
        adapters::sail_to_miralis(sail_ctx),
        "wfi instruction emulation is not correct"
    );
}

fn generate_csr_register() -> u64 {
    // We want only 12 bits
    let mut csr: u64 = any!(u64) & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x0;
    }

    // Odd pmpcfg indices configs are not allowed
    if 0x3A0 <= csr && csr <= 0x3AF {
        csr &= !0b1;
    }

    return csr;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn fences() {
    {
        let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

        let rs1 = any!(usize) & 0b11111;
        let rs2 = any!(usize) & 0b11111;

        ctx.emulate_sfence_vma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

        execute_SFENCE_VMA(
            &mut sail_ctx,
            BitVector::new(rs1 as u64),
            BitVector::new(rs2 as u64),
        );

        assert_eq!(
            ctx,
            adapters::sail_to_miralis(sail_ctx),
            "sfence-vma instruction emulation is not correct"
        );
    }
    {
        let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

        let rs1 = any!(usize) & 0b11111;
        let rs2 = any!(usize) & 0b11111;

        ctx.emulate_hfence_vvma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

        execute_HFENCE_VVMA(
            &mut sail_ctx,
            BitVector::new(rs1 as u64),
            BitVector::new(rs2 as u64),
        );

        assert_eq!(
            ctx,
            adapters::sail_to_miralis(sail_ctx),
            "hfence-vvma instruction emulation is not correct"
        );
    }
    {
        let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

        let rs1 = any!(usize) & 0b11111;
        let rs2 = any!(usize) & 0b11111;

        ctx.emulate_hfence_gvma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

        execute_HFENCE_GVMA(
            &mut sail_ctx,
            BitVector::new(rs1 as u64),
            BitVector::new(rs2 as u64),
        );

        assert_eq!(
            ctx,
            adapters::sail_to_miralis(sail_ctx),
            "hfence-gvma instruction emulation is not correct"
        );
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn read_csr() {
    let (mut ctx, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Infinite number of pmps for the formal verification
    ctx.nb_pmp = usize::MAX;

    let csr_register = generate_csr_register();

    // Read value from Miralis
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    let miralis_value = ctx.get(decoded_csr);

    // Read value from Sail
    let sail_value = readCSR(&mut sail_ctx, BitVector::<12>::new(csr_register)).bits as usize;

    // Verify value is the same
    assert_eq!(miralis_value, sail_value, "Read equivalence");
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn write_csr() {
    let mut csr_register = generate_csr_register();

    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate a random value
    let mut value_to_write: usize = any!(usize);

    // Write register in Miralis context
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    ctx.set_csr(decoded_csr, value_to_write, &mut mctx);

    if csr_register == 0b001100000011 {
        value_to_write |= mie::MIDELEG_READ_ONLY_ONE;
        value_to_write &= !mie::MIDELEG_READ_ONLY_ZERO;
    }

    // Write register in Sail context
    writeCSR(
        &mut sail_ctx,
        BitVector::<12>::new(csr_register),
        BitVector::<64>::new(value_to_write as u64),
    );

    // Pmp registers
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.pmpaddr,
        ctx.csr.pmpaddr,
        "Write pmp addr equivalence"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.pmpcfg,
        ctx.csr.pmpcfg,
        "Write pmp cfg equivalence"
    );

    // Verified and working
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mvendorid,
        ctx.csr.mvendorid,
        "Write mvendorid"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mimpid,
        ctx.csr.mimpid,
        "Write mimpid"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).hart_id,
        ctx.hart_id,
        "Write hart_id"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mconfigptr,
        ctx.csr.mconfigptr,
        "Write mconfigptr"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mtvec,
        ctx.csr.mtvec,
        "Write mtvec"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mscratch,
        ctx.csr.mscratch,
        "wWite mscratch"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mtval,
        ctx.csr.mtval,
        "Write mtval"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mcycle,
        ctx.csr.mcycle,
        "Write mcycle"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.minstret,
        ctx.csr.minstret,
        "Write minstret"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.tselect,
        ctx.csr.tselect,
        "Write tselect"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.stvec,
        ctx.csr.stvec,
        "Write stvec"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.sscratch,
        ctx.csr.sscratch,
        "Write sscratch"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.stval,
        ctx.csr.stval,
        "Write stval"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.satp,
        ctx.csr.satp,
        "Write satp"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.senvcfg,
        ctx.csr.senvcfg,
        "Write senvcfg"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.scause,
        ctx.csr.scause,
        "Write scause"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mcause,
        ctx.csr.mcause,
        "Write mcause"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mepc,
        ctx.csr.mepc,
        "Write mepc"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vstart,
        ctx.csr.vstart,
        "Write vstart"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.menvcfg,
        ctx.csr.menvcfg,
        "Write menvcfg"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mcountinhibit,
        ctx.csr.mcountinhibit,
        "Write mcountinhibit"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.medeleg,
        ctx.csr.medeleg,
        "Write medeleg"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vxsat,
        ctx.csr.vxsat,
        "Write vxssat"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vxrm,
        ctx.csr.vxrm,
        "Write vxrm"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vcsr,
        ctx.csr.vcsr,
        "Write vcsr"
    );
    assert_eq!(sail_to_miralis(sail_ctx).csr.vl, ctx.csr.vl, "Write vl");
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vtype,
        ctx.csr.vtype,
        "Write vtype"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.vlenb,
        ctx.csr.vlenb,
        "Write vlenb"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.sepc,
        ctx.csr.sepc,
        "Write sepc"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.misa,
        ctx.csr.misa,
        "Write misa"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mideleg,
        ctx.csr.mideleg,
        "Write mideleg"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mcounteren,
        ctx.csr.mcounteren,
        "Write mcountern"
    );
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.scounteren,
        ctx.csr.scounteren,
        "Write scounteren"
    );
    assert_eq!(sail_to_miralis(sail_ctx).csr.mip, ctx.csr.mip, "Write mip");
    assert_eq!(sail_to_miralis(sail_ctx).csr.mie, ctx.csr.mie, "Write mie");
    assert_eq!(
        sail_to_miralis(sail_ctx).csr.mstatus,
        ctx.csr.mstatus,
        "Write mstatus"
    );

}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn interrupt_virtualization() {
    let (mut ctx, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Check the virtualization
    step_interrupts_only(&mut sail_ctx, 0);
    ctx.check_and_inject_interrupts();

    // Verify the results
    assert_eq!(
        ctx,
        sail_to_miralis(sail_ctx),
        "Interrupt virtualisation doesn't work properly"
    )
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn pmp_equivalence() {
    let (_, _, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generation of the entire address space we want to check
    let address_to_check = BitVector::new(any!(u64) >> 4);

    // The virtual firmware is always running in userspace
    let virtual_firmware_privilege = Privilege::User;

    let access_type: AccessType = match any!(u8) % 4 {
        0 => AccessType::Read(()),
        1 => AccessType::Write(()),
        2 => AccessType::ReadWrite(((), ())),
        _ => AccessType::Execute(()),
    };

    let virtual_offset = VIRTUAL_PMP_OFFSET;
    let number_virtual_pmps = sys_pmp_count(()) - virtual_offset;

    // Deactivate the last pmp entries in the physical context
    for idx in number_virtual_pmps..sys_pmp_count(()) {
        sail_ctx.pmpcfg_n[idx] = BitField::new(0);
        sail_ctx.pmpaddr_n[idx] = BitVector::new(0);
    }

    let physical_check: Option<ExceptionType> = pmpCheck::<64>(
        &mut sail_ctx,
        address_to_check,
        8,
        access_type,
        virtual_firmware_privilege,
    );

    let virtual_check: Option<ExceptionType> = {
        // Creation of the PMP group
        let mut pmp_group = PmpGroup::new(sys_pmp_count(()));
        pmp_group.virt_pmp_offset = virtual_offset;

        // Physical write of the pmp registers
        pmp_group.load_with_offset(
            &pmpaddr_sail_to_miralis(sail_ctx.pmpaddr_n),
            &pmpcfg_sail_to_miralis(sail_ctx.pmpcfg_n),
            virtual_offset,
            number_virtual_pmps,
        );
        unsafe {
            write_pmp(&pmp_group).flush();
        }

        // Retrieve hardware context
        let mut generated_sail_ctx = adapters::miralis_to_sail(&return_userspace_ctx());

        // Execution of the pmp check function
        pmpCheck::<64>(
            &mut generated_sail_ctx,
            address_to_check,
            8,
            access_type,
            virtual_firmware_privilege,
        )
    };

    // Check pmp equivalence
    assert_eq!(
        physical_check, virtual_check,
        "pmp are not installed correctly in Miralis"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_csr_decoder() {
    let (_, mctx, _) = symbolic::new_symbolic_contexts();

    let register = any!(usize) % 0xFFF;

    // Decode values
    let decoded_value_sail = decode_csr_register(BitVector::new(register as u64));
    let decoded_value_miralis = mctx.decode_csr(register);

    assert_eq!(
        decoded_value_sail, decoded_value_miralis,
        "CSR Decoding is not similar"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_decoder() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u32);

    // Decode values
    let decoded_value_sail = ast_to_miralis_instr(encdec_backwards(
        &mut sail_ctx,
        BitVector::new(instr as u64),
    ));
    let decoded_value_miralis = mctx.decode_illegal_instruction(instr as usize);

    // For the moment, we ignore the values that are not decoded by the sail reference
    if decoded_value_sail != Instr::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_miralis,
            "decoders are not equivalent"
        );
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate instruction to decode and emulate
    let mut instr: usize = any!(u32) as usize;

    let decoded_instruction = mctx.decode_illegal_instruction(instr);

    let decoded_sail_instruction = encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    if decoded_instruction == Instr::Unknown {
        assert_eq!(decoded_sail_instruction, ast::ILLEGAL(BitVector::new(0)))
    }

    // For the moment, we verify the behavior only for MRET and WFI
    instr = match instr {
        // MRET
        0b00110000001000000000000001110011 => 0b00110000001000000000000001110011,
        // SRET
        0b00010000001000000000000001110011 => 0b00010000001000000000000001110011,
        // WFI
        _ => 0b00110000001000000000000001110011,
    };

    // Emulate instruction in Miralis
    ctx.emulate_illegal_instruction(&mut mctx, instr);

    // Execute value in sail
    execute::execute_ast(&mut sail_ctx, instr);

    // Check the equivalence
    assert_eq!(
        ctx,
        sail_to_miralis(sail_ctx),
        "emulation of privileged instructions isn't equivalent"
    );
}
