use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::return_userspace_ctx;
use miralis::arch::{mie, write_pmp, MCause, Register};
use miralis::decoder::IllegalInstruction;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use sail_decoder::decoder_illegal::sail_decoder_illegal;
use sail_decoder::decoder_load::sail_decoder_load;
use sail_decoder::decoder_store::sail_decoder_store;
use sail_model::{
    ast, execute_HFENCE_GVMA, execute_HFENCE_VVMA, execute_MRET, execute_SFENCE_VMA, execute_SRET,
    execute_WFI, pmpCheck, readCSR, set_next_pc, step_interrupts_only, trap_handler, writeCSR,
    AccessType, ExceptionType, Privilege,
};
use sail_prelude::{sys_pmp_count, BitField, BitVector};

use crate::adapters::{
    ast_to_miralis_instr, ast_to_miralis_load, ast_to_miralis_store, decode_csr_register,
    miralis_to_sail, pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis, sail_to_miralis,
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
        adapters::sail_to_miralis(sail_ctx, &mctx),
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
        adapters::sail_to_miralis(sail_ctx, &mctx),
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
        ctx.csr,
        adapters::sail_to_miralis(sail_ctx, &mctx).csr,
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
            adapters::sail_to_miralis(sail_ctx, &mctx),
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
            adapters::sail_to_miralis(sail_ctx, &mctx),
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
            adapters::sail_to_miralis(sail_ctx, &mctx),
            "hfence-gvma instruction emulation is not correct"
        );
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn read_csr() {
    let (ctx, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

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

    let is_mideleg = csr_register == 0b001100000011;

    // TODO: Handle the last few registers
    if is_mideleg {
        csr_register = 0;
    }

    // Generate a random value
    let mut value_to_write: usize = any!(usize);

    // Write register in Miralis context
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    ctx.set_csr(decoded_csr, value_to_write, &mut mctx);

    assert_eq!(
        sail_ctx.cur_privilege,
        Privilege::Machine,
        "not the correct precondition"
    );

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

    assert_eq!(
        sail_to_miralis(sail_ctx, &mctx).csr,
        ctx.csr,
        "Write equivalence"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn interrupt_virtualization() {
    let (mut ctx, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    sail_ctx.mideleg = BitField::new(0);
    ctx.csr.mideleg = 0;

    // Check the virtualization
    step_interrupts_only(&mut sail_ctx, 0);
    ctx.check_and_inject_interrupts();

    // Verify the results
    assert_eq!(
        ctx,
        sail_to_miralis(sail_ctx, &mctx),
        "Interrupt virtualisation doesn't work properly"
    )
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn exception_virtualization() {
    let (mut ctx, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    let trap_cause = generate_exception_cause();

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, &mctx, MCause::new(trap_cause as usize));

    // Emulate jump to trap handler in Miralis
    ctx.emulate_jump_trap_handler();

    // Emulate jump to trap handler in Sail
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

    let mut sail_ctx_generated = adapters::sail_to_miralis(sail_ctx, &mctx);
    // Update some meta-data maintained by Miralis
    sail_ctx_generated.is_wfi = ctx.is_wfi.clone();
    sail_ctx_generated.trap_info = ctx.trap_info.clone();

    assert_eq!(
        sail_ctx_generated, ctx,
        "Injection of trap doesn't work properly"
    );
}

/// Returns an arbitrary exception cause among the exceptions known to Miralis.
fn generate_exception_cause() -> usize {
    let code = any!(usize) & 0xF;
    if MCause::new(code) == MCause::UnknownException {
        0
    } else {
        code
    }
}

/// Simulate a hardware trap to fill the `trap_info` in the [VirtContext] with valid valued.
///
/// The simulation is done by instantiating a new Sail context and emulating a trap using the Sail
/// emulator.
fn fill_trap_info_structure(ctx: &mut VirtContext, mctx: &MiralisContext, cause: MCause) {
    let mut sail_ctx = miralis_to_sail(ctx);

    // Inject through the Sail emulator.
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

    let new_miralis_ctx = sail_to_miralis(sail_ctx, mctx);

    ctx.trap_info.mcause = new_miralis_ctx.csr.mcause;
    ctx.trap_info.mstatus = new_miralis_ctx.csr.mstatus;
    ctx.trap_info.mtval = new_miralis_ctx.csr.mtval;
    ctx.trap_info.mepc = new_miralis_ctx.csr.mepc;
    ctx.trap_info.mip = new_miralis_ctx.csr.mip;
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
    let instr = any!(u32, 0x30001073);

    // Decode values
    let decoded_value_sail = ast_to_miralis_instr(sail_decoder_illegal::encdec_backwards(
        &mut sail_ctx,
        BitVector::new(instr as u64),
    ));
    let decoded_value_miralis = mctx.decode_illegal_instruction(instr as usize);

    // For the moment, we ignore the values that are not decoded by the sail reference
    if decoded_value_sail != IllegalInstruction::Unknown {
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

    // For the moment, we verify the behavior only for MRET and WFI
    instr = match instr {
        // MRET
        0b00110000001000000000000001110011 => 0b00110000001000000000000001110011,
        // WFI
        _ => 0b00110000001000000000000001110011,
    };

    // Emulate instruction in Miralis
    ctx.emulate_illegal_instruction(&mut mctx, instr);

    // Execute value in sail
    execute::execute_ast(&mut sail_ctx, instr);

    // Check the equivalence
    assert_eq!(
        ctx.csr.mstatus,
        sail_to_miralis(sail_ctx, &mut mctx).csr.mstatus,
        "emulation of privileged instructions isn't equivalent"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_compressed_loads() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u16, 0x01073) & !0b11;

    let intermediate_sail_value =
        sail_decoder_load::encdec_compressed_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    match intermediate_sail_value {
        ast::ILLEGAL(_) => {}
        _ => {
            // Decode values
            let decoded_value_sail = ast_to_miralis_load(intermediate_sail_value);
            let decoded_value_miralis = mctx.decode_load(instr as usize);

            assert_eq!(
                decoded_value_sail, decoded_value_miralis,
                "decoders for compressed loads are not equivalent"
            );
        }
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_load() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u32, 0x01073) | 0b11;

    let intermediate_sail_value =
        sail_decoder_load::encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    match intermediate_sail_value {
        ast::ILLEGAL(_) => {}
        _ => {
            let decoded_value_sail = ast_to_miralis_load(intermediate_sail_value);

            let decoded_value_miralis = mctx.decode_load(instr as usize);

            assert_eq!(
                decoded_value_sail, decoded_value_miralis,
                "decoders for loads are not equivalent"
            );
        }
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_compressed_stores() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u16, 0x01073) & !0b11;

    let intermediate_sail_value = sail_decoder_store::encdec_compressed_backwards(
        &mut sail_ctx,
        BitVector::new(instr as u64),
    );

    match intermediate_sail_value {
        ast::ILLEGAL(_) => {}
        _ => {
            let decoded_value_sail =
                ast_to_miralis_store(sail_decoder_store::encdec_compressed_backwards(
                    &mut sail_ctx,
                    BitVector::new(instr as u64),
                ));

            let decoded_value_miralis = mctx.decode_store(instr as usize);

            assert_eq!(
                decoded_value_sail, decoded_value_miralis,
                "decoders for compressed stores are not equivalent"
            );
        }
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_stores() {
    let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u32, 0x01073) | 0b11;

    let intermediate_sail_value =
        sail_decoder_store::encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

    match intermediate_sail_value {
        ast::ILLEGAL(_) => {}
        _ => {
            // Decode values
            let decoded_value_sail = ast_to_miralis_store(intermediate_sail_value);

            let decoded_value_miralis = mctx.decode_store(instr as usize);

            assert_eq!(
                decoded_value_sail, decoded_value_miralis,
                "decoders for loads are not equivalent"
            );
        }
    }
}
