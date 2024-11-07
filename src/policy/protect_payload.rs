//! The protect payload policy, it protects the payload.

use miralis_core::abi_protect_payload;

use crate::arch::pmp::pmpcfg;
use crate::arch::pmp::pmplayout::POLICY_OFFSET;
use crate::arch::{Arch, Architecture, MCause, Register};
use crate::decoder::Instr;
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The protect payload policy module, which allow the payload to protect himself from the firmware at some point in time and enfore a boundary between the two components.
pub struct ProtectPayloadPolicy {
    protected: bool,
    general_register: [usize; 32],
    rules: [ForwardingRule; ForwardingRule::NB_RULES],
    last_cause: MCause,
}

impl PolicyModule for ProtectPayloadPolicy {
    fn init() -> Self {
        ProtectPayloadPolicy {
            protected: false,
            general_register: [0; 32],
            rules: ForwardingRule::build_forwarding_rules(),
            // It is important to let the first mode be EcallFromSMode as the firmware passes some information to the OS.
            // Setting this last_cause allows to pass the arguments during the first call.
            last_cause: MCause::EcallFromSMode,
        }
    }
    fn name() -> &'static str {
        "Protect Payload Policy"
    }

    fn ecall_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        self.check_interrupt(ctx, mctx)
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        self.check_interrupt(ctx, mctx)
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Clear general purpose registers
        let trap_cause = MCause::try_from(ctx.trap_info.mcause).unwrap();
        let filter_rule = ForwardingRule::match_rule(trap_cause, &mut self.rules);

        for i in 0..self.general_register.len() {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if !filter_rule.allow_in[i] {
                ctx.regs[i] = 0;
            }
        }

        // Lock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, 0x80400000);
        mctx.pmp
            .set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::NO_PERMISSIONS);

        self.last_cause = trap_cause;
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        let register_filter = ForwardingRule::match_rule(self.last_cause, &mut self.rules);

        // Restore general purpose registers
        for i in 0..self.general_register.len() {
            if !register_filter.allow_out[i] {
                ctx.regs[i] = self.general_register[i];
            }
        }

        // Unlock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, 0x80400000);
        mctx.pmp.set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::RWX);
    }

    const NUMBER_PMPS: usize = 2;
}

impl ProtectPayloadPolicy {
    fn check_interrupt(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) -> PolicyHookResult {
        let cause = ctx.trap_info.get_cause();
        match cause {
            MCause::LoadAddrMisaligned | MCause::StoreAddrMisaligned => {
                self.handle_misaligned_op(ctx, mctx)
            }
            MCause::EcallFromSMode => {
                if !self.is_policy_call(ctx) {
                    return PolicyHookResult::Ignore;
                }

                log::info!("Locking payload from payload");
                self.lock(mctx, ctx);
                ctx.pc += 4;
                PolicyHookResult::Overwrite
            }
            _ => PolicyHookResult::Ignore,
        }
    }

    fn lock(&mut self, _mctx: &mut MiralisContext, _ctx: &mut VirtContext) {
        self.protected = true;
    }

    fn is_policy_call(&mut self, ctx: &VirtContext) -> bool {
        let policy_eid: bool =
            ctx.get(Register::X17) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_EID;
        let lock_fid: bool =
            ctx.get(Register::X16) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_LOCK_FID;

        policy_eid && lock_fid
    }

    fn handle_misaligned_op(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) -> PolicyHookResult {
        let instr = unsafe { Arch::get_raw_faulting_instr(&ctx.trap_info) };
        let instr = mctx.decode(instr);
        match instr {
            Instr::Load {
                rd,
                rs1,
                imm,
                len,
                is_compressed,
                is_unsigned,
            } => {
                assert!(
                    !is_compressed,
                    "Implement support for compressed instructions"
                );
                assert!(!is_unsigned, "Implement support for unsigned instructions");
                assert!(
                    len.to_bytes() == 8,
                    "Implement support for other than 8 bytes misalinged accesses"
                );

                // Build the value
                let start_addr: *const u8 =
                    ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *const u8;

                let emulated_value: u64 = 0x0;
                let addr_emulated_value = &emulated_value as *const u64 as *mut u8;

                for idx in 0..len.to_bytes() {
                    unsafe {
                        *(addr_emulated_value.add(idx)) = *(start_addr.add(idx));
                    }
                }

                // Return the value
                ctx.regs[rd as usize] = emulated_value as usize;
            }
            Instr::Store {
                rs2,
                rs1,
                imm,
                len,
                is_compressed,
            } => {
                assert!(
                    !is_compressed,
                    "Implement support for compressed instructions"
                );
                assert!(
                    len.to_bytes() == 8,
                    "Implement support for other than 8 bytes misalinged accesses"
                );

                // Build the value
                let start_addr: *mut u8 =
                    ((ctx.regs[rs1 as usize] as isize + imm) as usize) as *mut u8;

                let emulated_value: u64 = 0x0;
                let addr_emulated_value = &emulated_value as *const u64 as *mut u8;

                for idx in 0..len.to_bytes() {
                    unsafe {
                        *(addr_emulated_value.add(idx)) = *(start_addr.add(idx));
                    }
                }

                // Return the value
                ctx.regs[rs2 as usize] = emulated_value as usize;
            }
            _ => {
                panic!("Must be a load instruction here")
            }
        }

        ctx.pc += 4;
        PolicyHookResult::Overwrite
    }
}

// ———————————————————————————————— Explicit Forwarding Rules ———————————————————————————————— //

#[derive(Clone)]
pub struct ForwardingRule {
    mcause: MCause,
    allow_in: [bool; 32],
    allow_out: [bool; 32],
}

impl ForwardingRule {
    pub const NB_RULES: usize = 1;

    fn match_rule(trap_cause: MCause, rules: &mut [ForwardingRule; 1]) -> ForwardingRule {
        for rule in rules {
            if trap_cause == rule.mcause {
                return rule.clone();
            }
        }

        Self::new_allow_nothing(trap_cause)
    }

    fn build_forwarding_rules() -> [ForwardingRule; Self::NB_RULES] {
        let mut rules = [Self::new_allow_nothing(MCause::EcallFromSMode); 1];

        // Build Ecall rule
        rules[0]
            .allow_register_in(Register::X10)
            .allow_register_in(Register::X11)
            .allow_register_in(Register::X12)
            .allow_register_in(Register::X13)
            .allow_register_in(Register::X14)
            .allow_register_in(Register::X15)
            .allow_register_in(Register::X16)
            .allow_register_in(Register::X17)
            .allow_register_out(Register::X10)
            .allow_register_out(Register::X11);

        rules
    }

    #[allow(unused)]
    fn new_allow_nothing(mcause: MCause) -> Self {
        ForwardingRule {
            mcause,
            allow_in: [false; 32],
            allow_out: [false; 32],
        }
    }

    fn allow_register_in(&mut self, reg: Register) -> &mut Self {
        self.allow_in[reg as usize] = true;
        self
    }

    fn allow_register_out(&mut self, reg: Register) -> &mut Self {
        self.allow_out[reg as usize] = true;
        self
    }
}
