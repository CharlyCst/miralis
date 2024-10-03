//! The protect payload policy, it protects the payload.

use miralis_core::abi_protect_payload;

use crate::arch::pmp::{build_napot, pmpcfg};
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
        for i in 0..32 {
            self.general_register[i] = ctx.regs[i];
            // We don't clear ecall registers
            if self.clear_register(i, ctx) {
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
        for i in 0..32 {
            // 10 & 11 are return registers
            if self.restore_register(i, ctx) {
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

    fn clear_register(&mut self, idx: usize, ctx: &mut VirtContext) -> bool {
        !(ctx.trap_info.get_cause() == MCause::EcallFromSMode && (10..18).contains(&idx))
    }

    fn restore_register(&mut self, idx: usize, ctx: &mut VirtContext) -> bool {
        !(ctx.trap_info.get_cause() == MCause::EcallFromSMode && (10..12).contains(&idx))
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
