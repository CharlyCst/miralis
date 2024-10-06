//! The protect payload policy, it protects the payload.

use core::slice;
use core::sync::atomic::{AtomicBool, Ordering};

use miralis_core::abi_protect_payload;
use tiny_keccak::{Hasher, Sha3};

use crate::arch::pmp::pmpcfg;
use crate::arch::pmp::pmplayout::POLICY_OFFSET;
use crate::arch::MCause::EcallFromSMode;
use crate::arch::{MCause, Register};
use crate::config::PAYLOAD_HASH_SIZE;
use crate::host::MiralisContext;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::{RegisterContextGetter, VirtContext};

const LINUX_LOCK_PAYLOAD_HASH: [u8; 32] = [
    241, 90, 158, 184, 200, 210, 145, 178, 30, 80, 200, 161, 56, 120, 75, 241, 68, 38, 21, 2, 248,
    112, 128, 155, 31, 240, 37, 94, 203, 66, 243, 167,
];

const TEST_POLICY_PAYLOAD: [u8; 32] = [
    138, 39, 74, 50, 74, 113, 151, 180, 78, 91, 92, 25, 132, 84, 192, 119, 38, 36, 19, 147, 193,
    166, 149, 149, 158, 179, 237, 5, 158, 54, 245, 69,
];

static FIRST_JUMP: AtomicBool = AtomicBool::new(true);

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
        // Step 1: Restore general purpose registers
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

        // Attempt to set `flag` to false only if it is currently true
        if FIRST_JUMP
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // Lock memory from all cores
            // TODO: add a proper barrier to ensure synchronization
            Plat::broadcast_policy_interrupt();

            let hashed_value = hash_payload(PAYLOAD_HASH_SIZE, ctx.pc);

            let not_linux_payload = hashed_value != LINUX_LOCK_PAYLOAD_HASH;
            let not_test_payload = hashed_value != TEST_POLICY_PAYLOAD;

            if not_linux_payload && not_test_payload {
                log::error!("Loaded payload is suspicious");
                log::error!("Hashed value: {:?}", hashed_value);
                log::error!("Expected value: {:?}", LINUX_LOCK_PAYLOAD_HASH);
                panic!("Protect Payload policy: Invalid hash");
            }
        }
    }

    // In this policy module, if we receive an interrupt from Miralis, it implies we need to lock the memory
    fn on_interrupt(&mut self, _ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        // Lock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, 0x80400000);
        mctx.pmp
            .set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::NO_PERMISSIONS);
    }

    const NUMBER_PMPS: usize = 2;
}

impl ProtectPayloadPolicy {
    fn lock(&mut self, _mctx: &mut MiralisContext, _ctx: &mut VirtContext) {
        self.protected = true;
    }

    fn is_policy_call(&mut self, ctx: &VirtContext) -> bool {
        ctx.get(Register::X17) == abi_protect_payload::MIRALIS_PROTECT_PAYLOAD_EID
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

// ———————————————————————————————— Hash primitive ———————————————————————————————— //

fn hash_payload(size_to_hash: usize, pc_start: usize) -> [u8; 32] {
    let payload_start: usize = 0x80400000;
    let payload_end: usize = 0x80400000 + size_to_hash;

    assert!(
        payload_start <= payload_end,
        "Invalid memory range for payload hashing"
    );

    unsafe {
        let mut hasher = Sha3::v256();

        let payload_content =
            slice::from_raw_parts(payload_start as *const u8, payload_end - payload_start);
        hasher.update(payload_content);

        hasher.update(&pc_start.to_le_bytes());

        let mut hashed_value = [0u8; 32];
        hasher.finalize(&mut hashed_value);

        hashed_value
    }
}
