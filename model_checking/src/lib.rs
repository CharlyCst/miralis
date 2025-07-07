use miralis::arch::pmp::pmplayout::VIRTUAL_PMP_OFFSET;
use miralis::arch::pmp::PmpGroup;
use miralis::arch::userspace::SOFT_CORE;
use miralis::arch::{csr, mie, write_pmp, MCause, Register};
use miralis::decoder::IllegalInst;
use miralis::host::MiralisContext;
use miralis::virt::traits::{HwRegisterContextSetter, RegisterContextGetter};
use miralis::virt::VirtContext;
use softcore_rv64::prelude::{bv, BitVector};
use softcore_rv64::raw;
use softcore_rv64::raw::{regidx, AccessType, Minterrupts, Pmpcfg_ent, Privilege};

use crate::adapters::{
    ast_to_miralis_instr, ast_to_miralis_load, ast_to_miralis_store, decode_csr_register,
    miralis_to_rv_core, pmpaddr_sail_to_miralis, pmpcfg_sail_to_miralis, rv_core_to_miralis,
};

#[macro_use]
mod symbolic;
mod adapters;
mod execute;
mod model;

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn mret() {
    let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    ctx.emulate_mret(&mut mctx);
    model::execute_MRET(&mut core);

    assert_eq!(
        ctx,
        adapters::rv_core_to_miralis(core, &mctx),
        "mret instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn sret() {
    let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    ctx.emulate_sret(&mut mctx);
    model::execute_SRET(&mut core);

    assert_eq!(
        ctx,
        adapters::rv_core_to_miralis(core, &mctx),
        "sret instruction emulation is not correct"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn wfi() {
    let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    ctx.emulate_wfi(&mut mctx);
    model::execute_WFI(&mut core);

    // This field is used only in Miralis. We set it to false otherwise the assertions fails.
    ctx.is_wfi = false;

    assert_eq!(
        ctx.csr,
        adapters::rv_core_to_miralis(core, &mctx).csr,
        "wfi instruction emulation is not correct"
    );
}

fn generate_csr_register() -> u64 {
    // We want only 12 bits
    let mut csr: u64 = any!(u64, 0x340) & 0xFFF;

    // Ignore sedeleg and sideleg
    if csr == 0b000100000010 || csr == 0b000100000011 {
        csr = 0x0;
    }
    // Some CSRs are not supported by the spec
    if csr == csr::SCONTEXT as u64 {
        csr = 0;
    }
    // And some others return non-deterministic values
    if csr == csr::CYCLE as u64
        || csr == csr::INSTRET as u64
        || csr == csr::MINSTRET as u64
        || csr == csr::MCYCLE as u64
        || csr == csr::TIME as u64
    {
        csr = 0;
    }

    // Odd pmpcfg indices configs are not allowed
    if (csr::PMPCFG0..=csr::PMPCFG15).contains(&(csr as usize)) {
        csr &= !0b1;
    }

    csr
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn fences() {
    {
        let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

        let rs1 = any!(usize) & 0b11111;
        let rs2 = any!(usize) & 0b11111;

        ctx.emulate_sfence_vma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

        model::execute_SFENCE_VMA(&mut core, regidx::new(rs1 as u8), regidx::new(rs2 as u8));

        assert_eq!(
            ctx,
            adapters::rv_core_to_miralis(core, &mctx),
            "sfence-vma instruction emulation is not correct"
        );
    }
    // {
    //     let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    //     let rs1 = any!(usize) & 0b11111;
    //     let rs2 = any!(usize) & 0b11111;

    //     ctx.emulate_hfence_vvma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

    //     execute_HFENCE_VVMA(
    //         &mut core,
    //         BitVector::new(rs1 as u64),
    //         BitVector::new(rs2 as u64),
    //     );

    //     assert_eq!(
    //         ctx,
    //         adapters::sail_to_miralis(core, &mctx),
    //         "hfence-vvma instruction emulation is not correct"
    //     );
    // }
    // {
    //     let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    //     let rs1 = any!(usize) & 0b11111;
    //     let rs2 = any!(usize) & 0b11111;

    //     ctx.emulate_hfence_gvma(&mut mctx, &Register::from(rs1), &Register::from(rs2));

    //     execute_HFENCE_GVMA(
    //         &mut core,
    //         BitVector::new(rs1 as u64),
    //         BitVector::new(rs2 as u64),
    //     );

    //     assert_eq!(
    //         ctx,
    //         adapters::sail_to_miralis(core, &mctx),
    //         "hfence-gvma instruction emulation is not correct"
    //     );
    // }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn read_csr() {
    let (ctx, mctx, mut core) = symbolic::new_symbolic_contexts();

    let csr_register = generate_csr_register();

    // Read value from Miralis
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    let miralis_value = ctx.get(decoded_csr);

    // Read value from Sail
    match core.get_csr(csr_register) {
        Some(sail_value) => {
            // Verify value is the same
            assert_eq!(
                miralis_value, sail_value as usize,
                "Miralis emulation returned a different value from the reference core"
            );
        }
        None => {
            // For all invalid CSRs Miralis returns 0
            assert_eq!(miralis_value, 0, "Invalid register should return 0");
        }
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn write_csr() {
    let mut csr_register = generate_csr_register();

    let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

    let is_mideleg = csr_register == 0b001100000011;

    // TODO: Handle the last few registers
    if is_mideleg {
        csr_register = 0;
    }

    // Generate a random value
    let mut value_to_write = any!(usize);

    // Write register in Miralis context
    let decoded_csr = mctx.decode_csr(csr_register as usize);
    ctx.set_csr(decoded_csr, value_to_write, &mut mctx);

    assert_eq!(
        core.cur_privilege,
        Privilege::Machine,
        "not the correct precondition"
    );

    if csr_register == 0b001100000011 {
        value_to_write |= mie::MIDELEG_READ_ONLY_ONE;
        value_to_write &= !mie::MIDELEG_READ_ONLY_ZERO;
    }

    // Write register in Sail context
    core.set_csr(csr_register, value_to_write as u64);

    assert_eq!(
        rv_core_to_miralis(core, &mctx).csr,
        ctx.csr,
        "Write equivalence"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn interrupt_virtualization() {
    let (mut ctx, mctx, mut core) = symbolic::new_symbolic_contexts();

    // We don't delegate any interrupts in the formal verification
    core.mideleg = Minterrupts { bits: bv(0) };
    ctx.csr.mideleg = 0;

    // Check the virtualization
    core.dispatch_interrupt();
    ctx.check_and_inject_interrupts();

    // Verify the results
    assert_eq!(
        ctx,
        rv_core_to_miralis(core, &mctx),
        "Interrupt virtualisation doesn't work properly"
    )
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn exception_virtualization() {
    let (mut ctx, mctx, mut core) = symbolic::new_symbolic_contexts();

    let trap_cause = generate_exception_cause();

    // Generate the trap handler
    fill_trap_info_structure(&mut ctx, &mctx, MCause::new(trap_cause));

    // Emulate jump to trap handler in Miralis
    ctx.emulate_firmware_trap();

    // Emulate jump to trap handler in Sail
    let pc = core.PC;
    let new_pc = raw::trap_handler(
        &mut core,
        Privilege::Machine,
        false,
        BitVector::new(trap_cause as u64),
        pc,
        None,
        None,
    );
    raw::set_next_pc(&mut core, new_pc);

    let mut core_ctx_generated = adapters::rv_core_to_miralis(core, &mctx);
    // Update some meta-data maintained by Miralis
    core_ctx_generated.is_wfi = ctx.is_wfi;
    core_ctx_generated.trap_info = ctx.trap_info.clone();

    assert_eq!(
        core_ctx_generated, ctx,
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
    let mut sail_ctx = miralis_to_rv_core(ctx);

    // Inject through the Sail emulator.
    let pc_argument = sail_ctx.PC;
    raw::trap_handler(
        &mut sail_ctx,
        Privilege::Machine,
        false,
        BitVector::new(cause as u64),
        pc_argument,
        None,
        None,
    );

    let new_miralis_ctx = rv_core_to_miralis(sail_ctx, mctx);

    ctx.trap_info.mcause = new_miralis_ctx.csr.mcause;
    ctx.trap_info.mstatus = new_miralis_ctx.csr.mstatus;
    ctx.trap_info.mtval = new_miralis_ctx.csr.mtval;
    ctx.trap_info.mepc = new_miralis_ctx.csr.mepc;
    ctx.trap_info.mip = new_miralis_ctx.csr.mip;
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn pmp_equivalence() {
    let (_, _, mut reference_core) = symbolic::new_symbolic_contexts();

    // Generation of the entire address space we want to check
    let address_to_check = any!(u64) >> 4;

    // The virtual firmware is always running in userspace
    let virtual_firmware_privilege = Privilege::User;

    let access_type = match any!(u8) % 4 {
        0 => AccessType::Read(()),
        1 => AccessType::Write(()),
        2 => AccessType::ReadWrite(((), ())),
        _ => AccessType::InstructionFetch(()),
    };

    let virtual_offset = VIRTUAL_PMP_OFFSET;
    let nb_phys_pmp = raw::sys_pmp_count(&mut reference_core, ()) as usize;
    let nb_virtual_pmps = nb_phys_pmp - virtual_offset;

    // Deactivate the last pmp entries in the physical context
    for idx in nb_virtual_pmps..nb_phys_pmp {
        reference_core.pmpcfg_n[idx] = Pmpcfg_ent { bits: bv(0) };
        reference_core.pmpaddr_n[idx] = bv(0);
    }

    reference_core.set_mode(virtual_firmware_privilege);
    let physical_check = reference_core.pmp_check(address_to_check, access_type);

    let virtual_check = {
        // Creation of the PMP group
        let mut pmp_group = PmpGroup::new(nb_phys_pmp);
        pmp_group.virt_pmp_offset = virtual_offset;

        // Physical write of the pmp registers
        pmp_group.load_with_offset(
            &pmpaddr_sail_to_miralis(reference_core.pmpaddr_n),
            &pmpcfg_sail_to_miralis(reference_core.pmpcfg_n),
            virtual_offset,
            nb_virtual_pmps,
        );
        unsafe {
            write_pmp(&pmp_group).flush();
        }

        // Execution of the pmp check function using the core Miralis is executing on
        SOFT_CORE.with_borrow_mut(|miralis_core| {
            miralis_core.set_mode(virtual_firmware_privilege);
            miralis_core.pmp_check(address_to_check, access_type)
        })
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
    let (_, mctx, mut core) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u32, 0x30001073);

    // Decode values
    let decoded_value_sail = ast_to_miralis_instr(model::sail_decoder_illegal::encdec_backwards(
        &mut core,
        BitVector::new(instr as u64),
    ));
    let decoded_value_miralis = mctx.decode_illegal_instruction(instr as usize);

    // For the moment, we ignore the values that are not decoded by the sail reference
    if decoded_value_sail != IllegalInst::Unknown {
        assert_eq!(
            decoded_value_sail, decoded_value_miralis,
            "decoders are not equivalent"
        );
    }
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn formally_verify_emulation_privileged_instructions() {
    let (mut ctx, mut mctx, mut core) = symbolic::new_symbolic_contexts();

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
    execute::execute_ast(&mut core, instr);

    // Check the equivalence
    assert_eq!(
        ctx.csr.mstatus,
        rv_core_to_miralis(core, &mctx).csr.mstatus,
        "emulation of privileged instructions isn't equivalent"
    );
}

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_compressed_loads() {
    let (_, mctx, mut core) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u16, 0x01073) & !0b11;

    let intermediate_sail_value = model::sail_decoder_load::encdec_compressed_backwards(
        &mut core,
        BitVector::new(instr as u64),
    );

    match intermediate_sail_value {
        raw::ast::ILLEGAL(_) => {}
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

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
// pub fn verify_load() {
//     let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

//     // Generate an instruction to decode
//     let instr = any!(u32, 0x01073) | 0b11;

//     let intermediate_sail_value =
//         sail_decoder_load::encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

//     match intermediate_sail_value {
//         ast::ILLEGAL(_) => {}
//         _ => {
//             let decoded_value_sail = ast_to_miralis_load(intermediate_sail_value);

//             let decoded_value_miralis = mctx.decode_load(instr as usize);

//             assert_eq!(
//                 decoded_value_sail, decoded_value_miralis,
//                 "decoders for loads are not equivalent"
//             );
//         }
//     }
// }

#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
pub fn verify_compressed_stores() {
    let (_, mctx, mut core) = symbolic::new_symbolic_contexts();

    // Generate an instruction to decode
    let instr = any!(u16, 0x01073) & !0b11;

    let intermediate_sail_value = model::sail_decoder_store::encdec_compressed_backwards(
        &mut core,
        BitVector::new(instr as u64),
    );

    match intermediate_sail_value {
        raw::ast::ILLEGAL(_) => {}
        _ => {
            let decoded_value_sail =
                ast_to_miralis_store(model::sail_decoder_store::encdec_compressed_backwards(
                    &mut core,
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

// #[cfg_attr(kani, kani::proof)]
// #[cfg_attr(test, test)]
// pub fn verify_stores() {
//     let (_, mctx, mut sail_ctx) = symbolic::new_symbolic_contexts();

//     // Generate an instruction to decode
//     let instr = any!(u32, 0x01073) | 0b11;

//     let intermediate_sail_value =
//         sail_decoder_store::encdec_backwards(&mut sail_ctx, BitVector::new(instr as u64));

//     match intermediate_sail_value {
//         ast::ILLEGAL(_) => {}
//         _ => {
//             // Decode values
//             let decoded_value_sail = ast_to_miralis_store(intermediate_sail_value);

//             let decoded_value_miralis = mctx.decode_store(instr as usize);

//             assert_eq!(
//                 decoded_value_sail, decoded_value_miralis,
//                 "decoders for loads are not equivalent"
//             );
//         }
//     }
// }
