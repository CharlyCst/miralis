use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::MCause::IllegalInstr;
use miralis::arch::{mie, write_pmp, MCause, Register};
use miralis::decoder::Instr;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use sail_decoder::encdec_backwards;
use sail_model::{
    ast, check_CSR, execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA,
    execute_SRET, execute_WFI, pmpCheck, readCSR, set_next_pc, step_interrupts_only, trap_handler,
    writeCSR, AccessType, ExceptionType, Privilege, SailVirtCtx,
};
use sail_prelude::{sys_pmp_count, BitField, BitVector};

use crate::adapters::{ast_to_miralis_instr, miralis_to_sail, sail_to_miralis};

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
    let csr_register = generate_csr_register();

    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate a random value
    let mut value_to_write: usize = any!(usize);

    if csr_register == 0b001100000011 {
        value_to_write |= mie::MIDELEG_READ_ONLY_ONE;
        value_to_write &= !mie::MIDELEG_READ_ONLY_ZERO;
    }

    // Write register in Miralis context
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    ctx.set_csr(decoded_csr, value_to_write, &mut mctx);

    // Write register in Sail context
    writeCSR(
        &mut sail_ctx,
        BitVector::<12>::new(csr_register),
        BitVector::<64>::new(value_to_write as u64),
    );
    // assert_eq!(sail_to_miralis(sail_ctx).csr.mie, ctx.csr.mie, "mie");
    assert_eq!(sail_to_miralis(sail_ctx), ctx, "write equivalence");
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
        adapters::sail_to_miralis(sail_ctx),
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
            &adapters::pmpaddr_sail_to_miralis(sail_ctx.pmpaddr_n),
            &adapters::pmpcfg_sail_to_miralis(sail_ctx.pmpcfg_n),
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
    let decoded_value_sail = adapters::decode_csr_register(BitVector::new(register as u64));
    let decoded_value_miralis = mctx.decode_csr(register);

    assert_eq!(
        decoded_value_sail, decoded_value_miralis,
        "CSR Decoding is not similar"
    );
}



pub fn generate_csr_register_fancy(sail_virt_ctx: &mut SailVirtCtx, is_write: bool) -> u64 {
    // We want only 12 bits
    let mut csr: u64 = any!(u64) & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x341;
    }

    // Odd pmpcfg indices configs are not allowed
    if 0x3A0 <= csr && csr <= 0x3AF {
        csr &= !0b1;
    }

    if !check_CSR(
        sail_virt_ctx,
        BitVector::new(csr),
        Privilege::Machine,
        is_write,
    ) {
        csr = 0x341;
    }

    if 0x303 == csr {
        csr = 0x341;
    }

    return csr;
}

fn generate_raw_instruction(mctx: &mut MiralisContext, sail_virt_ctx: &mut SailVirtCtx) -> usize {
    const SYSTEM_MASK: u32 = 0b1110011;

    // Generate instruction to decode and emulate
    let mut instr: usize = ((any!(u32) & !0b1111111) | SYSTEM_MASK) as usize;

    // For the moment, we simply avoid the csr with illegal instructions, I will handle it in a second case
    instr = match mctx.decode_illegal_instruction(instr) {
        Instr::Csrrw { .. } | Instr::Csrrwi { .. } => {
            ((generate_csr_register_fancy(sail_virt_ctx, true) << 20) | (instr & 0xfffff) as u64)
                as usize
        }
        Instr::Csrrc { csr: _, rd: _, rs1 } | Instr::Csrrs { csr: _, rd: _, rs1 } => {
            ((generate_csr_register_fancy(sail_virt_ctx, rs1 != Register::X0) << 20)
                | (instr & 0xfffff) as u64) as usize
        }
        Instr::Csrrci {
            csr: _,
            rd: _,
            uimm,
        }
        | Instr::Csrrsi {
            csr: _,
            rd: _,
            uimm,
        } => {
            ((generate_csr_register_fancy(sail_virt_ctx, uimm != 0) << 20)
                | (instr & 0xfffff) as u64) as usize
        }
        _ => instr,
    };

    return instr;
}

fn generate_trap_cause() -> usize {
    let code = any!(usize) & 0xF;
    if MCause::new(code) == MCause::UnknownException {
        0
    } else {
        code
    }
}

fn fill_trap_info_structure(ctx: &mut VirtContext, cause: MCause) {
    let mut sail_ctx = miralis_to_sail(ctx);

    // Inject trap
    let pc_argument = sail_ctx.PC;
    trap_handler(
        &mut sail_ctx,
        Privilege::Machine,
        false,
        BitVector::new(cause as u64),
        pc_argument,
        None,
        None,
    );

    let new_miralis_ctx = sail_to_miralis(sail_ctx);

    ctx.trap_info.mcause = new_miralis_ctx.csr.mcause;
    ctx.trap_info.mstatus = new_miralis_ctx.csr.mstatus;
    ctx.trap_info.mtval = new_miralis_ctx.csr.mtval;
    ctx.trap_info.mepc = new_miralis_ctx.csr.mepc;
}

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
pub fn verify_trap_logic() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    let trap_cause = generate_trap_cause();

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, MCause::new(trap_cause as usize));

    ctx.emulate_jump_trap_handler();

    {
        let pc = sail_ctx.PC;
        let new_pc = trap_handler(
            &mut sail_ctx,
            Privilege::Machine,
            false,
            BitVector::new(trap_cause as u64),
            pc,
            None,
            None,
        );
        set_next_pc(&mut sail_ctx, new_pc);
    }

    let mut sail_ctx_generated = adapters::sail_to_miralis(sail_ctx);

    sail_ctx_generated.is_wfi = ctx.is_wfi.clone();
    sail_ctx_generated.trap_info = ctx.trap_info.clone();

    assert_eq!(
        sail_ctx_generated, ctx,
        "Injection of trap doesn't work properly"
    );
}

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, IllegalInstr);

    let instr = generate_raw_instruction(&mut mctx, &mut sail_ctx);

    // Decode the instructions
    let decoded_instruction = mctx.decode_illegal_instruction(instr);
    let decoded_sail_instruction = encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    let is_unknown_sail = decoded_sail_instruction == ast::ILLEGAL(BitVector::new(0));
    let is_unknown_miralis = decoded_instruction == Instr::Unknown;

    // assert_eq!(is_unknown_sail, is_unknown_miralis, "Both decoder don't decode the same instruction set");

    if !is_unknown_miralis {
        // assert_eq!(decoded_instruction, adapters::ast_to_miralis_instr(decoded_sail_instruction), "instruction are decoded not similar");

        // Emulate instruction in Miralis
        ctx.emulate_illegal_instruction(&mut mctx, instr);

        // Execute value in sail
        execute::execute_ast(&mut sail_ctx, instr);

        let mut sail_ctx_generated = adapters::sail_to_miralis(sail_ctx);

        // These fields are used only in the miralis context and are irrelevant
        sail_ctx_generated.is_wfi = ctx.is_wfi.clone();
        sail_ctx_generated.trap_info = ctx.trap_info.clone();

        assert_eq!(sail_ctx_generated, ctx, "Overall");
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_decoder() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = (any!(u32) & !0b1111111) | 0b1110011;

    // Decode values
    let decoded_value_sail = ast_to_miralis_instr(encdec_backwards(
        &mut sail_ctx,
        BitVector::new(instr as u64),
    ));
    let decoded_value_miralis = mctx.decode_illegal_instruction(instr as usize);

    // We verify the equivalence with the following decomposition
    // A <--> B <==> A --> B && B --> A

    // For the moment, we ignore the values that are not decoded by the sail reference
    if decoded_value_sail != Instr::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_miralis,
            "decoders are not equivalent"
        );
    }

    if decoded_value_miralis != Instr::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_sail,
            "decoders are not equivalent"
        )
    }
}
