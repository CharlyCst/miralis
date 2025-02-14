//! Emulation logic for misaligned loads and stores

use crate::arch::{parse_mpp_return_mode, Arch, Architecture, Csr};
use crate::decoder::Instr;
use crate::host::MiralisContext;
use crate::policy::PolicyHookResult;
use crate::virt::VirtContext;

pub fn emulate_misaligned_read(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
) -> PolicyHookResult {
    let instr_ptr = ctx.trap_info.mepc as *const u8;

    let mut instr: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    copy_from_previous_mode(instr_ptr, &mut instr);

    let instr = mctx.decode_load(u64::from_le_bytes(instr) as usize);

    match instr {
        Instr::Load {
            rd, rs1, imm, len, ..
        } => {
            // Build the value
            let start_addr: *const u8 =
                ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *const u8;

            match len.to_bytes() {
                8 => {
                    let mut value_to_read: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                    copy_from_previous_mode(start_addr, &mut value_to_read);

                    // Return the value
                    ctx.regs[rd as usize] = u64::from_le_bytes(value_to_read) as usize;
                }
                4 => {
                    let mut value_to_read: [u8; 4] = [0, 0, 0, 0];

                    copy_from_previous_mode(start_addr, &mut value_to_read);

                    // Return the value
                    ctx.regs[rd as usize] = u32::from_le_bytes(value_to_read) as usize;
                }
                2 => {
                    let mut value_to_read: [u8; 2] = [0, 0];

                    copy_from_previous_mode(start_addr, &mut value_to_read);

                    // Return the value
                    ctx.regs[rd as usize] = u16::from_le_bytes(value_to_read) as usize;
                }
                _ => {
                    unreachable!("Misaligned read with a unexpected byte length")
                }
            }
        }
        _ => {
            unreachable!("Must be a load instruction here")
        }
    }

    ctx.pc += 4;
    PolicyHookResult::Overwrite
}

pub fn emulate_misaligned_write(
    ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
) -> PolicyHookResult {
    let instr_ptr = ctx.trap_info.mepc as *const u8;

    let mut instr: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    copy_from_previous_mode(instr_ptr, &mut instr);

    let instr = mctx.decode_store(u64::from_le_bytes(instr) as usize);

    match instr {
        Instr::Store {
            rs2, rs1, imm, len, ..
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
            }
        }
        _ => {
            unreachable!("Must be a load instruction here")
        }
    }

    ctx.pc += 4;
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
