//! The protect payload policy, it protects the payload.

use miralis_core::abi_protect_payload;

use crate::arch::pmp::{build_napot, pmpcfg};
use crate::arch::MCause::EcallFromSMode;
use crate::arch::{Arch, Architecture, MCause, Register};
use crate::host::MiralisContext;
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

/// The protect payload policy module, which allow the payload to protect himself from the firmware at some point in time and enfore a boundary between the two components.
pub struct ProtectPayloadPolicy {
    protected: bool,
    general_register: [usize; 32],
    confidential_values_firmware: ConfidentialCSRs,
    confidential_values_payload: ConfidentialCSRs,
    rules: [ForwardingRule; ForwardingRule::NB_RULES],
}

impl PolicyModule for ProtectPayloadPolicy {
    fn init() -> Self {
        ProtectPayloadPolicy {
            protected: false,
            general_register: [0; 32],
            confidential_values_firmware: ConfidentialCSRs {
                stvec: 0,
                scounteren: 0,
                senvcfg: 0,
                sscratch: 0,
                sepc: 0,
                scause: 0,
                stval: 0,
                satp: 0,
            },
            confidential_values_payload: ConfidentialCSRs {
                stvec: 0,
                scounteren: 0,
                senvcfg: 0,
                sscratch: 0,
                sepc: 0,
                scause: 0,
                stval: 0,
                satp: 0,
            },
            rules: ForwardingRule::build_forwarding_rules(),
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
        if !self.is_policy_call(ctx) {
            return PolicyHookResult::Ignore;
        }

        if ctx.get(Register::X16) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_LOCK_FID {
            log::info!("Locking payload from firmware");
            self.lock(mctx, ctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        if !self.is_policy_call(ctx) {
            return PolicyHookResult::Ignore;
        }

        if ctx.get(Register::X16) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_LOCK_FID {
            log::info!("Locking payload from payload");
            self.lock(mctx, ctx);
            ctx.pc += 4;
            PolicyHookResult::Overwrite
        } else {
            PolicyHookResult::Ignore
        }
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        if !self.protected {
            return;
        }

        // Step 1: Clear general purpose registers
        let trap_cause = MCause::try_from(ctx.trap_info.mcause).unwrap();
        let register_filter = ForwardingRule::match_rule(trap_cause, &mut self.rules);

        for i in 0..self.general_register.len() {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if i == 10 {
                log::warn!("Allowed {:?}", register_filter.allow_in);
            }
            if !register_filter.allow_in[i] {
                ctx.regs[i] = 0;
            }
        }

        // Step 2: Save sensitives privileged registers
        self.confidential_values_payload = get_confidential_values(ctx);
        self.write_confidential_registers(ctx, false);

        // Step 3: Lock memory
        mctx.pmp.set(
            mctx.devices.len() + 2,
            build_napot(0x80400000, 0x80000).unwrap(),
            pmpcfg::NAPOT,
        );

        unsafe {
            Arch::write_pmp(&mctx.pmp).flush();
        }
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        if !self.protected {
            return;
        }

        // Step 1: Restore general purpose registers
        let trap_cause = MCause::try_from(ctx.trap_info.mcause).unwrap();
        let register_filter = ForwardingRule::match_rule(trap_cause, &mut self.rules);

        for i in 0..self.general_register.len() {
            if !register_filter.allow_out[i] {
                ctx.regs[i] = self.general_register[i];
            }
        }

        // Step 2: Restore sensitives privileged registers
        self.write_confidential_registers(ctx, true);

        // Step 3: Unlock memory
        mctx.pmp.set(mctx.devices.len() + 2, 0, pmpcfg::INACTIVE);

        unsafe {
            Arch::write_pmp(&mctx.pmp).flush();
        }
    }
}

// TODO: Check what to do with fcsr & sip & cycle
pub struct ConfidentialCSRs {
    pub stvec: usize,
    pub scounteren: usize,
    pub senvcfg: usize,
    pub sscratch: usize,
    pub sepc: usize,
    pub scause: usize,
    pub stval: usize,
    pub satp: usize,
}

fn get_confidential_values(ctx: &mut VirtContext) -> ConfidentialCSRs {
    ConfidentialCSRs {
        stvec: ctx.csr.stvec,
        scounteren: ctx.csr.scounteren,
        senvcfg: ctx.csr.senvcfg,
        sscratch: ctx.csr.sscratch,
        sepc: ctx.csr.sepc,
        scause: ctx.csr.scause,
        stval: ctx.csr.stval,
        satp: ctx.csr.satp,
    }
}

impl ProtectPayloadPolicy {
    fn lock(&mut self, _mctx: &mut MiralisContext, ctx: &mut VirtContext) {
        // Step 1: Get view of confidential registers
        self.confidential_values_firmware = get_confidential_values(ctx);
        self.confidential_values_payload = get_confidential_values(ctx);

        // Step 2: Mark as protected
        self.protected = true;
    }

    fn is_policy_call(&mut self, ctx: &VirtContext) -> bool {
        ctx.get(Register::X17) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_EID
    }

    fn write_confidential_registers(&mut self, ctx: &mut VirtContext, restore: bool) {
        if restore {
            ctx.csr.stvec = self.confidential_values_payload.stvec;
            ctx.csr.scounteren = self.confidential_values_payload.scounteren;
            ctx.csr.senvcfg = self.confidential_values_payload.senvcfg;
            ctx.csr.sscratch = self.confidential_values_payload.sscratch;
            ctx.csr.sepc = self.confidential_values_payload.sepc;
            ctx.csr.scause = self.confidential_values_payload.scause;
            ctx.csr.stval = self.confidential_values_payload.stval;
            ctx.csr.satp = self.confidential_values_payload.satp;
        } else {
            ctx.csr.stvec = self.confidential_values_firmware.stvec;
            ctx.csr.scounteren = self.confidential_values_firmware.scounteren;
            ctx.csr.senvcfg = self.confidential_values_firmware.senvcfg;
            ctx.csr.sscratch = self.confidential_values_firmware.sscratch;
            ctx.csr.sepc = self.confidential_values_firmware.sepc;
            ctx.csr.scause = self.confidential_values_firmware.scause;
            ctx.csr.stval = self.confidential_values_firmware.stval;
            ctx.csr.satp = self.confidential_values_firmware.satp;
        }
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
       log::warn!("Matching rules");
        for idx in 0..rules.len() {
            log::warn!("{:?} vs {:?}", trap_cause, rules[idx].mcause);
            if trap_cause == rules[idx].mcause {
                return rules[idx].clone();
            }
        }

        Self::new_allow_nothing(trap_cause)
    }

    fn build_forwarding_rules() -> [ForwardingRule; Self::NB_RULES] {
        let mut rules = [Self::new_allow_nothing(EcallFromSMode); 1];

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
            mcause: mcause,
            allow_in: [false; 32],
            allow_out: [false; 32],
        }
    }

    #[allow(unused)]
    fn new_allow_everything(mcause: MCause) -> Self {
        ForwardingRule {
            mcause: mcause,
            allow_in: [true; 32],
            allow_out: [true; 32],
        }
    }

    #[allow(unused)]
    fn allow_register_in(&mut self, reg: Register) -> &mut Self {
        self.allow_in[reg as usize] = true;
        self
    }

    #[allow(unused)]
    fn block_register_in(&mut self, reg: Register) -> &mut Self {
        self.allow_in[reg as usize] = false;
        self
    }

    #[allow(unused)]
    fn allow_register_out(&mut self, reg: Register) -> &mut Self {
        self.allow_out[reg as usize] = true;
        self
    }

    #[allow(unused)]
    fn block_register_out(&mut self, reg: Register) -> &mut Self {
        self.allow_out[reg as usize] = false;
        self
    }
}
