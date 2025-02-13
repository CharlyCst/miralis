//! The offload policy is a special policy used to reduce the number of world switches
//! It emulates the misaligned loads and stores, the read of the "time" register and offlaods the timer extension handling in Miralis

use miralis_core::sbi_codes;

use crate::arch::{
    get_raw_faulting_instr, parse_mpp_return_mode, Arch, Architecture, Csr, MCause, Register,
};
use crate::decoder::Instr;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::traits::{RegisterContextGetter, RegisterContextSetter};
use crate::virt::VirtContext;

pub struct OffloadPolicy {}

impl PolicyModule for OffloadPolicy {
    fn init() -> Self {
        OffloadPolicy {}
    }

    fn name() -> &'static str {
        "Offload Policy"
    }

    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        let trap_info = ctx.trap_info.clone();

        match trap_info.get_cause() {
            MCause::LoadAddrMisaligned => self.emulate_misaligned_read(ctx, mctx),
            MCause::StoreAddrMisaligned => self.emulate_misaligned_write(ctx, mctx),
            MCause::EcallFromSMode => {
                let timer_eid: bool = ctx.get(Register::X17) == sbi_codes::SBI_TIMER_EID;
                let timer_fid: bool = ctx.get(Register::X16) == sbi_codes::SBI_TIMER_FID;

                if timer_eid && timer_fid {
                    let v_clint = Plat::get_vclint();
                    v_clint.write_clint_from_payload(ctx, mctx, ctx.regs[Register::X10 as usize]);
                    ctx.pc += 4;
                    PolicyHookResult::Overwrite
                } else {
                    PolicyHookResult::Ignore
                }
            }
            MCause::IllegalInstr => {
                let instr = unsafe { get_raw_faulting_instr(&ctx.trap_info) };

                let is_privileged_op: bool = instr & 0x7f == 0b111_0011;
                let is_time_register: bool = (instr >> 20) == 0b1100_0000_0001;

                if is_privileged_op && is_time_register {
                    let rd = (instr >> 7) & 0b11111;
                    let _rs1 = (instr >> 15) & 0b11111;

                    let func3_mask = instr & 0b111000000000000;
                    match func3_mask {
                        0x2000 => {
                            ctx.set(Register::try_from(rd).unwrap(), Arch::read_csr(Csr::Time));
                            ctx.pc += 4;
                            return PolicyHookResult::Overwrite;
                        }
                        0x1000 | 0x3000 | 0x5000 | 0x6000 | 0x7000 => {
                            todo!("Handle the offload of other CSR instructions")
                        }
                        _ => {}
                    }
                }

                PolicyHookResult::Ignore
            }
            _ => PolicyHookResult::Ignore,
        }
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        _ctx: &mut VirtContext,
        _mctx: &mut MiralisContext,
    ) {
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        _ctx: &mut VirtContext,
        _mctx: &mut MiralisContext,
    ) {
    }

    fn on_interrupt(&mut self, _ctx: &mut VirtContext, _mctx: &mut MiralisContext) {}

    const NUMBER_PMPS: usize = 0;
}

impl OffloadPolicy {
    fn emulate_misaligned_read(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) -> PolicyHookResult {
        let instr_ptr = ctx.trap_info.mepc as *const u8;

        let mut instr: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        unsafe {
            self.copy_from_previous_mode(instr_ptr, &mut instr);
        }

        match mctx.decode_load(u64::from_le_bytes(instr) as usize) {
            Instr::Load {
                rd, rs1, imm, len, ..
            } => {
                // Build the value
                let start_addr: *const u8 =
                    ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *const u8;

                match len.to_bytes() {
                    8 => {
                        let mut value_to_read: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                        unsafe {
                            self.copy_from_previous_mode(start_addr, &mut value_to_read);
                        }

                        // Return the value
                        ctx.regs[rd as usize] = u64::from_le_bytes(value_to_read) as usize;
                    }
                    4 => {
                        let mut value_to_read: [u8; 4] = [0, 0, 0, 0];
                        unsafe {
                            self.copy_from_previous_mode(start_addr, &mut value_to_read);
                        }

                        // Return the value
                        ctx.regs[rd as usize] = u32::from_le_bytes(value_to_read) as usize;
                    }
                    2 => {
                        let mut value_to_read: [u8; 2] = [0, 0];
                        unsafe {
                            self.copy_from_previous_mode(start_addr, &mut value_to_read);
                        }

                        // Return the value
                        ctx.regs[rd as usize] = u16::from_le_bytes(value_to_read) as usize;
                    }
                    _ => {
                        todo!("Implement support for other than 2,4,8 bytes misalinged accesses, current size: {}", len.to_bytes())
                    }
                }
            }
            _ => {
                panic!("Must be a load instruction here")
            }
        }

        ctx.pc += 4;
        PolicyHookResult::Overwrite
    }

    fn emulate_misaligned_write(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) -> PolicyHookResult {
        let instr_ptr = ctx.trap_info.mepc as *const u8;

        let mut instr: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        unsafe {
            self.copy_from_previous_mode(instr_ptr, &mut instr);
        }

        match mctx.decode_store(u64::from_le_bytes(instr) as usize) {
            Instr::Store {
                rs2, rs1, imm, len, ..
            } => {
                assert!(
                    len.to_bytes() == 8 || len.to_bytes() == 4 || len.to_bytes() == 2,
                    "Implement support for other than 2,4,8 bytes misalinged accesses"
                );

                if rs1 as usize != ((rs1 as usize) as isize) as usize {
                    panic!("Conversion is weird")
                }

                // Build the value
                let start_addr: *mut u8 =
                    ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *mut u8;

                match len.to_bytes() {
                    8 => {
                        let val = ctx.regs[rs2 as usize] as u64;
                        let mut value_to_store: [u8; 8] = val.to_le_bytes();

                        unsafe {
                            self.copy_from_previous_mode_store(&mut value_to_store, start_addr);
                        }
                    }
                    4 => {
                        let val = ctx.regs[rs2 as usize] as u32;
                        let mut value_to_store: [u8; 4] = val.to_le_bytes();

                        unsafe {
                            self.copy_from_previous_mode_store(&mut value_to_store, start_addr);
                        }
                    }
                    2 => {
                        let val = ctx.regs[rs2 as usize] as u16;
                        let mut value_to_store: [u8; 2] = val.to_le_bytes();

                        unsafe {
                            self.copy_from_previous_mode_store(&mut value_to_store, start_addr);
                        }
                    }
                    _ => {
                        todo!("Implement support for other than 2,4,8 bytes misalinged accesses, current size: {}", len.to_bytes())
                    }
                }
            }
            _ => {
                panic!("Must be a load instruction here")
            }
        }

        ctx.pc += 4;
        PolicyHookResult::Overwrite
    }

    unsafe fn copy_from_previous_mode(&mut self, src: *const u8, dest: &mut [u8]) {
        // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
        let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
        unsafe { Arch::read_bytes_from_mode(src, dest, mode).unwrap() }
    }

    unsafe fn copy_from_previous_mode_store(&mut self, src: &mut [u8], dest: *mut u8) {
        // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
        let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
        unsafe { Arch::store_bytes_from_mode(src, dest, mode).unwrap() }
    }
}
