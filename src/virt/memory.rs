//! Emulation logic for misaligned loads and stores

use crate::arch::{Arch, Architecture, get_raw_faulting_instr, parse_mpp_return_mode};
use crate::decoder::{LoadInstr, StoreInstr};
use crate::host::MiralisContext;
use crate::virt::VirtContext;

pub fn emulate_misaligned_read(ctx: &mut VirtContext, mctx: &mut MiralisContext) -> Result<(), ()> {
    let raw_instruction = unsafe { get_raw_faulting_instr(ctx) };
    let mode = parse_mpp_return_mode(ctx.trap_info.mstatus);
    let success;

    let LoadInstr {
        rd,
        rs1,
        imm,
        len,
        is_compressed,
        ..
    } = mctx.decode_load(raw_instruction);

    assert!(
        len.to_bytes() == 8 || len.to_bytes() == 4 || len.to_bytes() == 2,
        "Implement support for other than 2,4,8 bytes misaligned accesses"
    );

    // Build the value
    let start_addr: *const u8 = ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *const u8;

    ctx.regs[rd as usize] = match len.to_bytes() {
        8 => {
            let mut value_to_read: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            success = unsafe { Arch::read_bytes_from_mode(start_addr, &mut value_to_read, mode) };
            u64::from_le_bytes(value_to_read) as usize
        }
        4 => {
            let mut value_to_read: [u8; 4] = [0, 0, 0, 0];
            success = unsafe { Arch::read_bytes_from_mode(start_addr, &mut value_to_read, mode) };
            u32::from_le_bytes(value_to_read) as usize
        }
        2 => {
            let mut value_to_read: [u8; 2] = [0, 0];
            success = unsafe { Arch::read_bytes_from_mode(start_addr, &mut value_to_read, mode) };
            u16::from_le_bytes(value_to_read) as usize
        }
        _ => {
            unreachable!("Misaligned read with an unexpected byte length")
        }
    };

    match success {
        Ok(_) => {
            ctx.pc += if is_compressed { 2 } else { 4 };
            Ok(())
        }
        Err(_) => Err(()),
    }
}

pub fn emulate_misaligned_write(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
) -> Result<(), ()> {
    let raw_instruction = unsafe { get_raw_faulting_instr(ctx) };
    let mode = parse_mpp_return_mode(ctx.trap_info.mstatus);
    let success;

    let StoreInstr {
        rs2,
        rs1,
        imm,
        len,
        is_compressed,
    } = mctx.decode_store(raw_instruction);

    assert!(
        len.to_bytes() == 8 || len.to_bytes() == 4 || len.to_bytes() == 2,
        "Implement support for other than 2,4,8 bytes misaligned accesses"
    );

    // Build the value
    let start_addr: *mut u8 = ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *mut u8;

    match len.to_bytes() {
        8 => {
            let val = ctx.regs[rs2 as usize] as u64;
            let mut value_to_store: [u8; 8] = val.to_le_bytes();
            success = unsafe { Arch::store_bytes_from_mode(&mut value_to_store, start_addr, mode) };
        }
        4 => {
            let val = ctx.regs[rs2 as usize] as u32;
            let mut value_to_store: [u8; 4] = val.to_le_bytes();
            success = unsafe { Arch::store_bytes_from_mode(&mut value_to_store, start_addr, mode) };
        }
        2 => {
            let val = ctx.regs[rs2 as usize] as u16;
            let mut value_to_store: [u8; 2] = val.to_le_bytes();
            success = unsafe { Arch::store_bytes_from_mode(&mut value_to_store, start_addr, mode) };
        }
        _ => {
            unreachable!("Misaligned write with an unexpected byte length")
        }
    };

    match success {
        Ok(_) => {
            ctx.pc += if is_compressed { 2 } else { 4 };
            Ok(())
        }
        Err(_) => Err(()),
    }
}
