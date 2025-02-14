//! Emulation logic for misaligned loads and stores

use crate::arch::{get_raw_faulting_instr, parse_mpp_return_mode, Arch, Architecture, Csr};
use crate::decoder::Instr;
use crate::host::MiralisContext;
use crate::policy::PolicyHookResult;
use crate::virt::VirtContext;

pub fn emulate_misaligned_read(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
) -> PolicyHookResult {
    let raw_instruction = unsafe { get_raw_faulting_instr(&ctx.trap_info) };

    match mctx.decode_load(raw_instruction) {
        Instr::Load {
            rd,
            rs1,
            imm,
            len,
            is_compressed,
            ..
        } => {
            assert!(
                len.to_bytes() == 8 || len.to_bytes() == 4 || len.to_bytes() == 2,
                "Implement support for other than 2,4,8 bytes misalinged accesses"
            );

            // Build the value
            let start_addr: *const u8 =
                ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *const u8;

            ctx.regs[rd as usize] = match len.to_bytes() {
                8 => {
                    let mut value_to_read: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                    copy_from_previous_mode(start_addr, &mut value_to_read);
                    u64::from_le_bytes(value_to_read) as usize
                }
                4 => {
                    let mut value_to_read: [u8; 4] = [0, 0, 0, 0];
                    copy_from_previous_mode(start_addr, &mut value_to_read);
                    u32::from_le_bytes(value_to_read) as usize
                }
                2 => {
                    let mut value_to_read: [u8; 2] = [0, 0];
                    copy_from_previous_mode(start_addr, &mut value_to_read);
                    u16::from_le_bytes(value_to_read) as usize
                }
                _ => {
                    unreachable!("Misaligned read with a unexpected byte length")
                }
            };

            ctx.pc += if is_compressed { 2 } else { 4 }
        }
        _ => {
            unreachable!("Must be a load instruction here")
        }
    }

    PolicyHookResult::Overwrite
}

pub fn emulate_misaligned_write(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
) -> PolicyHookResult {
    let raw_instruction = unsafe { get_raw_faulting_instr(&ctx.trap_info) };

    match mctx.decode_store(raw_instruction) {
        Instr::Store {
            rs2,
            rs1,
            imm,
            len,
            is_compressed,
        } => {
            assert!(
                len.to_bytes() == 8 || len.to_bytes() == 4 || len.to_bytes() == 2,
                "Implement support for other than 2,4,8 bytes misalinged accesses"
            );

            // Build the value
            let start_addr: *mut u8 = ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *mut u8;

            match len.to_bytes() {
                8 => {
                    let val = ctx.regs[rs2 as usize] as u64;
                    let mut value_to_store: [u8; 8] = val.to_le_bytes();

                    copy_from_previous_mode_store(&mut value_to_store, start_addr);
                }
                4 => {
                    let val = ctx.regs[rs2 as usize] as u32;
                    let mut value_to_store: [u8; 4] = val.to_le_bytes();

                    copy_from_previous_mode_store(&mut value_to_store, start_addr);
                }
                2 => {
                    let val = ctx.regs[rs2 as usize] as u16;
                    let mut value_to_store: [u8; 2] = val.to_le_bytes();

                    copy_from_previous_mode_store(&mut value_to_store, start_addr);
                }
                _ => {
                    unreachable!("Misaligned write with a unexpected byte length")
                }
            };

            ctx.pc += if is_compressed { 2 } else { 4 }
        }
        _ => {
            unreachable!("Must be a load instruction here")
        }
    }

    PolicyHookResult::Overwrite
}

fn copy_from_previous_mode(src: *const u8, dest: &mut [u8]) {
    // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
    let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
    unsafe { Arch::read_bytes_from_mode(src, dest, mode).unwrap() }
}

fn copy_from_previous_mode_store(src: &mut [u8], dest: *mut u8) {
    // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
    let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
    unsafe { Arch::store_bytes_from_mode(src, dest, mode).unwrap() }
}
