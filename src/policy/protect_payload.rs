//! The protect payload policy, it protects the payload.
use core::slice;
use core::sync::atomic::{AtomicBool, Ordering};

use miralis_core::sbi_codes;
use miralis_core::sbi_codes::SBI_ERR_DENIED;
use tiny_keccak::{Hasher, Sha3};

use crate::arch::pmp::pmpcfg;
use crate::arch::pmp::pmplayout::POLICY_OFFSET;
use crate::arch::{get_raw_faulting_instr, mie, mstatus, MCause, Register, TrapInfo};
use crate::config::{PAYLOAD_HASH_SIZE, TARGET_PAYLOAD_ADDRESS};
use crate::host::MiralisContext;
use crate::logger;
use crate::platform::{Plat, Platform};
use crate::policy::{PolicyHookResult, PolicyModule};
use crate::virt::memory::{emulate_misaligned_read, emulate_misaligned_write};
use crate::virt::traits::*;
use crate::virt::{VirtContext, VirtCsr};

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
    need_to_restore_csr: bool,
    general_registers: [usize; 32],
    csr_registers: VirtCsr,
    forward_register_value_to_firmware: [bool; 32],
    forward_register_value_to_payload: [bool; 32],
}

impl PolicyModule for ProtectPayloadPolicy {
    fn init() -> Self {
        ProtectPayloadPolicy {
            need_to_restore_csr: false,
            general_registers: [0; 32],
            // The firmware must be able to pass information such as the device tree to the operating system. We allow all registers to pass during the first transition
            csr_registers: VirtCsr::new_empty(),
            forward_register_value_to_firmware: [true; 32],
            forward_register_value_to_payload: [true; 32],
        }
    }
    fn name() -> &'static str {
        "Protect Payload Policy"
    }

    fn trap_from_firmware(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        self.check_trap(ctx, mctx)
    }

    fn trap_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        self.check_trap(ctx, mctx)
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        self.forward_register_value_to_firmware = [false; 32];
        self.forward_register_value_to_payload = [false; 32];

        if ctx.trap_info.get_cause() == MCause::EcallFromSMode {
            // This function forwards only the registers necessary for the call determined by the (eid, fid) pair
            for i in 0..check_nb_registers_to_forward_per_eid_fid(
                ctx.get(Register::X17),
                ctx.get(Register::X16),
            ) {
                self.forward_register_value_to_firmware[Register::X10 as usize + i] = true;
            }

            // We pass the (eid, fid) pair to the firmware
            self.forward_register_value_to_firmware[Register::X16 as usize] = true;
            self.forward_register_value_to_firmware[Register::X17 as usize] = true;

            // We allow to the firmware to pass registers a0 & a1 in the case of an ecall
            self.forward_register_value_to_payload[Register::X10 as usize] = true;
            self.forward_register_value_to_payload[Register::X11 as usize] = true;
        }

        // If the illegal instruction is whitelisted, we allow every register to be modified
        if ctx.trap_info.get_cause() == MCause::IllegalInstr {
            self.check_illegal_instruction(&mut ctx.trap_info);
        }

        // The code becomes harder to read with the linter suggestion
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.general_registers.len() {
            self.general_registers[i] = ctx.regs[i];
            // Clear the value if not allowed to forward
            if !self.forward_register_value_to_firmware[i] {
                ctx.regs[i] = 0;
            }
        }

        // We hide the supervisor CSR registers to the firmware as they are sensitive
        self.clear_supervisor_csr(ctx);

        // Lock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, TARGET_PAYLOAD_ADDRESS);
        mctx.pmp
            .set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::NO_PERMISSIONS);
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // Restore general purpose registers
        for i in 0..self.general_registers.len() {
            // Restore registers where the value is not forwarded
            if !self.forward_register_value_to_payload[i] {
                ctx.regs[i] = self.general_registers[i];
            }
        }

        // Unlock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, TARGET_PAYLOAD_ADDRESS);
        mctx.pmp.set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::RWX);

        // We restore the supervisor csr registers
        self.restore_supervisor_csr(ctx);
        self.restore_supervisor_csr(ctx);

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

            // TODO: In the future, we should throw an error
            if not_linux_payload && not_test_payload {
                log::error!("Loaded payload is suspicious");
                log::error!("Hashed value: {:?}", hashed_value);
                log::error!("Expected value: {:?}", LINUX_LOCK_PAYLOAD_HASH);
                log::error!("Protect Payload policy: Invalid hash");
            }
        }
    }

    // In this policy module, if we receive an interrupt from Miralis, it implies we need to lock the memory
    fn on_interrupt(&mut self, _ctx: &mut VirtContext, mctx: &mut MiralisContext) {
        // Lock memory
        mctx.pmp.set_inactive(POLICY_OFFSET, TARGET_PAYLOAD_ADDRESS);
        mctx.pmp
            .set_tor(POLICY_OFFSET + 1, usize::MAX, pmpcfg::NO_PERMISSIONS);
    }

    const NUMBER_PMPS: usize = 2;
}

impl ProtectPayloadPolicy {
    fn check_trap(&mut self, ctx: &mut VirtContext, mctx: &mut MiralisContext) -> PolicyHookResult {
        let cause = ctx.trap_info.get_cause();
        match cause {
            MCause::LoadAddrMisaligned => emulate_misaligned_read(ctx, mctx),
            MCause::StoreAddrMisaligned => emulate_misaligned_write(ctx, mctx),
            // In the meantime, we must explicitly disable this feature to run Ubuntu with the protect payload policy
            MCause::EcallFromSMode
                if ctx.get(Register::X17) == sbi_codes::SBI_DEBUG_CONSOLE_EXTENSION_EID =>
            {
                logger::debug!("Ignoring console ecall to the debug_console_extension, please implement a bounce buffer if the feature is required");
                // Explicitly tell the payload this feature is not available
                ctx.set(Register::X10, SBI_ERR_DENIED);
                ctx.pc += 4;
                PolicyHookResult::Overwrite
            }
            _ => PolicyHookResult::Ignore,
        }
    }

    fn check_illegal_instruction(&mut self, trap_info: &mut TrapInfo) {
        let instr = unsafe { get_raw_faulting_instr(trap_info) };

        let is_privileged_op: bool = instr & 0x7f == 0b111_0011;
        let is_time_register: bool = (instr >> 20) == 0b1100_0000_0001;

        if is_privileged_op && is_time_register {
            match (instr >> 12) & 0b111 {
                1..=3 => {
                    let r1 = (instr >> 7) & 0b11111;
                    let r2 = (instr >> 15) & 0b11111;
                    self.forward_register_value_to_firmware[r1] = true;
                    self.forward_register_value_to_firmware[r2] = true;
                    self.forward_register_value_to_payload[r1] = true;
                    self.forward_register_value_to_payload[r2] = true;
                }
                5..=7 => {
                    let r1 = (instr >> 7) & 0b11111;
                    self.forward_register_value_to_firmware[r1] = true;
                    self.forward_register_value_to_payload[r1] = true;
                }
                _ => {}
            };
        }
    }

    const MIP_HIDE_MASK: usize = mie::SSIE_FILTER;
    const MIE_HIDE_MASK: usize = mie::SIE_FILTER;
    const MSTATUS_HIDE_MASK: usize = mstatus::MXR_FILTER
        | mstatus::SUM_FILTER
        | mstatus::XS_FILTER
        | mstatus::FS_FILTER
        | mstatus::VS_FILTER
        | mstatus::SD_FILTER
        | mstatus::SPP_FILTER
        | mstatus::SPIE_FILTER
        | mstatus::UPIE_FILTER
        | mstatus::SIE_FILTER
        | mstatus::SIE_OFFSET;

    // Saves and clear CSR registers
    fn clear_supervisor_csr(&mut self, ctx: &mut VirtContext) {
        self.need_to_restore_csr = true;

        self.csr_registers.stvec = ctx.csr.stvec;
        self.csr_registers.scounteren = ctx.csr.scounteren;
        self.csr_registers.scause = ctx.csr.scause;
        self.csr_registers.senvcfg = ctx.csr.senvcfg;
        self.csr_registers.sscratch = ctx.csr.sscratch;
        self.csr_registers.sepc = ctx.csr.sepc;
        self.csr_registers.scause = ctx.csr.scause;
        self.csr_registers.stval = ctx.csr.stval;
        self.csr_registers.satp = ctx.csr.satp;

        ctx.csr.stvec = 0;
        ctx.csr.scounteren = 0;
        ctx.csr.scause = 0;
        ctx.csr.senvcfg = 0;
        ctx.csr.sscratch = 0;
        ctx.csr.sepc = 0;
        ctx.csr.scause = 0;
        ctx.csr.stval = 0;
        ctx.csr.satp = 0;

        self.csr_registers.mip = ctx.csr.mip & Self::MIP_HIDE_MASK;
        ctx.csr.mip &= !Self::MIP_HIDE_MASK;

        self.csr_registers.mie = ctx.csr.mie & Self::MIE_HIDE_MASK;
        ctx.csr.mie &= !Self::MIE_HIDE_MASK;

        self.csr_registers.mstatus = ctx.csr.mstatus & Self::MSTATUS_HIDE_MASK;
        ctx.csr.mstatus &= !Self::MSTATUS_HIDE_MASK;
    }

    fn restore_supervisor_csr(&mut self, ctx: &mut VirtContext) {
        if self.need_to_restore_csr {
            ctx.csr.stvec = self.csr_registers.stvec;
            ctx.csr.scounteren = self.csr_registers.scounteren;
            ctx.csr.scause = self.csr_registers.scause;
            ctx.csr.senvcfg = self.csr_registers.senvcfg;
            ctx.csr.sscratch = self.csr_registers.sscratch;
            ctx.csr.sepc = self.csr_registers.sepc;
            ctx.csr.scause = self.csr_registers.scause;
            ctx.csr.stval = self.csr_registers.stval;
            ctx.csr.satp = self.csr_registers.satp;

            ctx.csr.mip = (ctx.csr.mip & !Self::MIP_HIDE_MASK) | self.csr_registers.mip;
            ctx.csr.mie = (ctx.csr.mie & !Self::MIE_HIDE_MASK) | self.csr_registers.mie;
            ctx.csr.mstatus =
                (ctx.csr.mstatus & !Self::MSTATUS_HIDE_MASK) | self.csr_registers.mstatus;
        }
    }
}

// ———————————————————————————————— Hash primitive ———————————————————————————————— //

fn hash_payload(size_to_hash: usize, pc_start: usize) -> [u8; 32] {
    let payload_start: usize = TARGET_PAYLOAD_ADDRESS;
    let payload_end: usize = TARGET_PAYLOAD_ADDRESS + size_to_hash;

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

// ———————————————————————————————— Filtering rules for ecall - automatically generated ———————————————————————————————— //

fn check_nb_registers_to_forward_per_eid_fid(eid: usize, fid: usize) -> usize {
    match (eid, fid) {
        (0x00, 0) => 1,
        (0x01, 0) => 1,
        (0x02, 0) => 0,
        (0x03, 0) => 0,
        (0x04, 0) => 1,
        (0x05, 0) => 1,
        (0x06, 0) => 3,
        (0x07, 0) => 4,
        (0x08, 0) => 0,
        (0x10, 0) => 0,
        (0x10, 1) => 0,
        (0x10, 2) => 0,
        (0x10, 3) => 1,
        (0x10, 4) => 0,
        (0x10, 5) => 0,
        (0x10, 6) => 0,
        (0x43505043, 0) => 1,
        (0x43505043, 1) => 1,
        (0x43505043, 2) => 1,
        (0x43505043, 3) => 2,
        (0x4442434E, 0) => 3,
        (0x4442434E, 1) => 3,
        (0x4442434E, 2) => 1,
        (0x44425452, 0) => 1,
        (0x44425452, 1) => 3,
        (0x44425452, 2) => 2,
        (0x44425452, 3) => 1,
        (0x44425452, 4) => 1,
        (0x44425452, 5) => 2,
        (0x44425452, 6) => 2,
        (0x44425452, 7) => 2,
        (0x46574654, 0) => 3,
        (0x46574654, 1) => 1,
        (0x48534D, 0) => 3,
        (0x48534D, 1) => 0,
        (0x48534D, 2) => 1,
        (0x48534D, 3) => 3,
        (0x4D505859, 0) => 0,
        (0x4D505859, 1) => 3,
        (0x4D505859, 2) => 1,
        (0x4D505859, 3) => 3,
        (0x4D505859, 4) => 3,
        (0x4D505859, 5) => 3,
        (0x4D505859, 6) => 3,
        (0x4D505859, 7) => 1,
        (0x4E41434C, 0) => 1,
        (0x4E41434C, 1) => 3,
        (0x4E41434C, 2) => 1,
        (0x4E41434C, 3) => 1,
        (0x4E41434C, 4) => 0,
        (0x504D55, 0) => 1,
        (0x504D55, 1) => 1,
        (0x504D55, 2) => 5,
        (0x504D55, 3) => 4,
        (0x504D55, 4) => 3,
        (0x504D55, 5) => 1,
        (0x504D55, 6) => 1,
        (0x504D55, 7) => 3,
        (0x504D55, 8) => 4,
        (0x52464E43, 0) => 2,
        (0x52464E43, 1) => 4,
        (0x52464E43, 2) => 5,
        (0x52464E43, 3) => 5,
        (0x52464E43, 4) => 4,
        (0x52464E43, 5) => 5,
        (0x52464E43, 6) => 4,
        (0x53525354, 0) => 2,
        (0x535345, 0) => 5,
        (0x535345, 1) => 5,
        (0x535345, 2) => 3,
        (0x535345, 3) => 1,
        (0x535345, 4) => 1,
        (0x535345, 5) => 1,
        (0x535345, 6) => 0,
        (0x535345, 7) => 2,
        (0x535345, 8) => 0,
        (0x535345, 9) => 0,
        (0x535441, 0) => 3,
        (0x53555350, 0) => 3,
        (0x54494D45, 0) => 1,
        (0x735049, 0) => 2,
        // This one is used in our test "test_protect_payload_payload" in the payload folder and is not part of the specification
        // The reason we introduce it is that there is no rule using all the registers currently and therefore we can't test the 6 register using one of the existing rules
        (0x8475bd0, 1) => 6,
        _ => {
            log::warn!("This eid, fid pair is unknown{:x} {:x}", eid, fid);
            0
        }
    }
}
