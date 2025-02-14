//! The offload policy is a special policy used to reduce the number of world switches
//! It emulates the misaligned loads and stores, the read of the "time" register and offlaods the timer extension handling in Miralis

use miralis_core::sbi_codes;

use crate::arch::{get_raw_faulting_instr, Arch, Architecture, Csr, MCause, Register};
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::rw_emulation::{emulate_misaligned_read, emulate_misaligned_write};
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
            MCause::LoadAddrMisaligned => emulate_misaligned_read(ctx, mctx),
            MCause::StoreAddrMisaligned => emulate_misaligned_write(ctx, mctx),
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

    const NUMBER_PMPS: usize = 0;
}
