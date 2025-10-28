//! The Keystone security policy
//!
//! This policy module enforces the Keystone policies, i.e. it enables the creation of user-level
//! enclaves by leveraging PMP for memory isolation.

use core::cmp::PartialEq;
use core::ptr;

use miralis_config::DELEGATE_PERF_COUNTER;

use crate::arch::perf_counters::DELGATE_PERF_COUNTERS_MASK;
use crate::arch::pmp::pmplayout::MODULE_OFFSET;
use crate::arch::pmp::{Segment, pmpcfg};
use crate::arch::{
    Arch, Architecture, Csr, MCause, Mode, Register, parse_mpp_return_mode, set_mpp, write_pmp,
};
use crate::host::MiralisContext;
use crate::modules::{Module, ModuleAction};
use crate::policy::keystone::ReturnCode::IllegalArgument;
use crate::virt::traits::*;
use crate::{RegisterContextGetter, VirtContext, logger};

/// Keystone parameters
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/platform/generic/platform.h#L11
const ENCL_MAX: usize = 1; // Maximum number of enclaves

/// Keystone EID & FIDs
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L5C1-L6C1
mod sbi {
    #![allow(unused)]
    pub const KEYSTONE_EID: usize = 0x08424b45;

    // 1999-2999 are called by host
    pub const CREATE_ENCLAVE_FID: usize = 2001;
    pub const DESTROY_ENCLAVE_FID: usize = 2002;
    pub const RUN_ENCLAVE_FID: usize = 2003;
    pub const RESUME_ENCLAVE_FID: usize = 2005;

    // 2999-3999 are called by enclave
    pub const RANDOM_FID: usize = 3001;
    pub const ATTEST_ENCLAVE_FID: usize = 3002;
    pub const GET_SEALING_KEY_FID: usize = 3003;
    pub const STOP_ENCLAVE_FID: usize = 3004;
    pub const EXIT_ENCLAVE_FID: usize = 3006;
}

/// Keystone return codes
///
/// See https://github.com/keystone-enclave/keystone/blob/master/sdk/include/shared/sm_err.h
#[derive(Debug)]
enum ReturnCode {
    Success = 0,
    EnclaveUnknownError = 100000,
    EnclaveInterrupted = 100002,
    EnclaveNotDestroyable = 100005,
    IllegalArgument = 100008,
    EnclaveNotResumable = 100010,
    EnclaveEdgeCallHost = 100011,
    NoFreeResources = 100013,
    EnclaveNotFresh = 100016,
    NotImplemented = 100100,
}

/// Arguments used to create a keystone enclave
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L59
#[repr(C)]
struct CreateArgs {
    epm_paddr: usize, // Enclave region
    epm_size: usize,
    utm_paddr: usize, // Untrusted region
    utm_size: usize,
    runtime_paddr: usize,
    user_paddr: usize,
    free_paddr: usize,
    free_requested: usize,
}

/// Reason for stopping an enclave
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sdk/include/shared/sm_call.h#L37
enum StoppedReason {
    Other = -1,
    Interrupt = 0,
    EdgeCallHost = 1,
}

impl From<usize> for StoppedReason {
    fn from(value: usize) -> Self {
        match value as isize {
            0 => StoppedReason::Interrupt,
            1 => StoppedReason::EdgeCallHost,
            _ => StoppedReason::Other,
        }
    }
}

/// Enclave definitions
///
/// See https://github.com/keystone-enclave/keystone/blob/80ffb2f9d4e774965589ee7c67609b0af051dc8b/sm/src/enclave.h
#[derive(Default, PartialEq)]
enum EnclaveState {
    #[default]
    Invalid = -1,
    Fresh,
    Stopped,
    Running,
}

#[derive(Default)]
struct EnclaveCtx {
    // General purpose registers
    regs: [usize; 32],
    pc: usize,
    // S-mode registers
    sie: usize,
    sip: usize,
    satp: usize,
    sepc: usize,
    stvec: usize,
    stval: usize,
    scause: usize,
    sstatus: usize,
    senvcfg: usize,
    sscratch: usize,
    scounteren: usize,
    // M-mode registers
    mideleg: usize,
    mpp: Mode,
}

impl EnclaveCtx {
    fn swap_ctx(&mut self, mctx: &MiralisContext, ctx: &mut VirtContext) {
        // Swap U-mode registers
        core::mem::swap(&mut self.regs, &mut ctx.regs);
        core::mem::swap(&mut self.pc, &mut ctx.pc);

        unsafe {
            // Swap S-mode registers
            if mctx.hw.available_reg.senvcfg {
                self.senvcfg = Arch::write_csr(Csr::Senvcfg, self.senvcfg);
            }
            self.sie = Arch::write_csr(Csr::Sie, self.sie);
            self.sip = Arch::write_csr(Csr::Sip, self.sip);
            self.satp = Arch::write_csr(Csr::Satp, self.satp);
            self.sepc = Arch::write_csr(Csr::Sepc, self.sepc);
            self.stvec = Arch::write_csr(Csr::Stvec, self.stvec);
            self.stval = Arch::write_csr(Csr::Stval, self.stval);
            self.scause = Arch::write_csr(Csr::Scause, self.scause);
            self.sstatus = Arch::write_csr(Csr::Sstatus, self.sstatus);
            self.sscratch = Arch::write_csr(Csr::Sscratch, self.sscratch);
            self.scounteren = Arch::write_csr(Csr::Scounteren, self.scounteren);

            // Swap M-mode registers
            self.mideleg = Arch::write_csr(Csr::Mideleg, self.mideleg);
            self.mpp = set_mpp(self.mpp);
        }
    }
}

#[derive(Default)]
struct Enclave {
    eid: usize,          // Enclave ID
    hart_id: usize,      // The ID of the hart that is running the enclave
    epm: Segment,        // The enclave physical memory region used by the enclave
    utm: Segment,        // The untrusted physical memory region shared by the enclave and the OS.
    state: EnclaveState, // State of the enclave
    ctx: EnclaveCtx,     // Enclave context
}

/// The keystone policy module
///
/// See https://keystone-enclave.org/
#[derive(Default)]
pub struct KeystonePolicy {
    enclaves: [Enclave; ENCL_MAX], // TODO: Accessing enclaves is not thread-safe
    nonce: usize,                  // TODO: Use a better rng
}

impl KeystonePolicy {
    /// Allocate an enclave slot and returns the index of the newly allocated enclave
    fn allocate_enclave(&mut self) -> Result<usize, ReturnCode> {
        for i in 0..ENCL_MAX {
            let enclave = &mut self.enclaves[i];
            if let EnclaveState::Invalid = enclave.state {
                return Ok(i);
            }
        }

        Err(ReturnCode::NoFreeResources)
    }

    /// Returns the enclave that was running before trapping in Miralis, or None if no enclave was running
    fn get_active_enclave(&mut self, ctx: &VirtContext) -> Option<&mut Enclave> {
        self.enclaves
            .iter_mut()
            .find(|e| e.state == EnclaveState::Running && e.hart_id == ctx.hart_id)
    }

    /// Configure PMPs so that the enclave cannot be accessed
    fn lock_enclave(mctx: &mut MiralisContext, enclave: &mut Enclave) {
        let pmp_id = MODULE_OFFSET + enclave.eid * 2;
        mctx.pmp.set_inactive(pmp_id, enclave.epm.start());
        mctx.pmp.set_tor(
            pmp_id + 1,
            enclave.epm.start() + enclave.epm.size(),
            pmpcfg::NO_PERMISSIONS,
        );

        unsafe {
            write_pmp(&mctx.pmp).flush();
        }
    }

    /// Configure PMPs so that only the enclave and the untrusted memory can be accessed
    fn unlock_enclave(mctx: &mut MiralisContext, enclave: &mut Enclave) {
        let pmp_id = MODULE_OFFSET + enclave.eid * 2;

        // Grant access to the enclave physical memory
        mctx.pmp.set_inactive(pmp_id, enclave.epm.start());
        mctx.pmp.set_tor(
            pmp_id + 1,
            enclave.epm.start() + enclave.epm.size(),
            pmpcfg::RWX,
        );

        // TODO: The enclave is currently allowed to access the payload memory, which it normally
        // shouldn't. This is a temporary compromise due to limitations in the number of available
        // PMPs (8 or fewer). Properly securing the payload would require additional PMPs.
        unsafe {
            write_pmp(&mctx.pmp).flush();
        }
    }

    fn context_switch_to_enclave(
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
        enclave: &mut Enclave,
    ) {
        enclave.state = EnclaveState::Running;
        enclave.hart_id = ctx.hart_id;
        enclave.ctx.swap_ctx(mctx, ctx);

        Self::unlock_enclave(mctx, enclave);
    }

    fn context_switch_to_host(
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
        enclave: &mut Enclave,
    ) {
        enclave.state = EnclaveState::Stopped;
        enclave.ctx.swap_ctx(mctx, ctx);

        Self::lock_enclave(mctx, enclave);
    }

    fn is_create_args_valid(&mut self, args: &mut CreateArgs) -> bool {
        // check if physical addresses are valid
        if args.epm_size == 0 {
            return false;
        }

        // check if overflow
        if args.epm_paddr >= args.epm_paddr + args.epm_size {
            return false;
        }
        if args.utm_paddr >= args.utm_paddr + args.utm_size {
            return false;
        }

        let epm_start = args.epm_paddr;
        let epm_end = args.epm_paddr + args.epm_size;

        // check if physical addresses are in the range
        if args.runtime_paddr < epm_start || args.runtime_paddr >= epm_end {
            return false;
        }

        if args.user_paddr < epm_start || args.user_paddr >= epm_end {
            return false;
        }

        if args.free_paddr < epm_start || args.free_paddr > epm_end {
            // note: free_paddr == epm_end if there's no free memory
            return false;
        }

        // check the order of physical addresses
        if args.runtime_paddr > args.user_paddr {
            return false;
        }

        if args.user_paddr > args.free_paddr {
            return false;
        }

        true
    }

    fn create_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Create enclave");

        // Copy the arguments from the S-mode virtual memory to the M-mode physical memory
        const ARGS_SIZE: usize = size_of::<CreateArgs>();
        let src = ctx.get(Register::X10) as *const u8;
        let mut dest: [u8; ARGS_SIZE] = [0; ARGS_SIZE];
        let mode = parse_mpp_return_mode(Arch::read_csr(Csr::Mstatus));
        let res = unsafe { Arch::read_bytes_from_mode(src, &mut dest, mode) };
        if res.is_err() {
            return ReturnCode::IllegalArgument;
        }

        let mut args = unsafe { ptr::read(dest.as_ptr() as *const CreateArgs) };
        if !self.is_create_args_valid(&mut args) {
            return IllegalArgument;
        }

        // Find a free enclave slot and initialize it
        let eid = match self.allocate_enclave() {
            Ok(index) => index,
            Err(code) => return code,
        };

        let enclave = &mut self.enclaves[eid];
        enclave.eid = eid;
        enclave.state = EnclaveState::Fresh;
        enclave.epm = Segment::new(args.epm_paddr, args.epm_size);
        enclave.utm = Segment::new(args.utm_paddr, args.utm_size);

        // Set initial enclave context
        enclave.ctx = EnclaveCtx::default();
        enclave.ctx.pc = args.epm_paddr - 4; // First instruction executed by the enclave.
        enclave.ctx.regs[11] = args.epm_paddr; // Enclave arguments
        enclave.ctx.regs[12] = args.epm_size;
        enclave.ctx.regs[13] = args.runtime_paddr;
        enclave.ctx.regs[14] = args.user_paddr;
        enclave.ctx.regs[15] = args.free_paddr;
        enclave.ctx.regs[16] = args.utm_paddr;
        enclave.ctx.regs[17] = args.utm_size;
        enclave.ctx.satp = 0; // Enclave uses physical addresses in the first run
        enclave.ctx.mideleg = 0; // No interrupts are delegated in the first run
        enclave.ctx.mpp = Mode::S; // Enclave starts in S-mode
        if DELEGATE_PERF_COUNTER {
            enclave.ctx.scounteren = DELGATE_PERF_COUNTERS_MASK;
        }

        Self::lock_enclave(mctx, enclave);

        ctx.set(Register::X11, eid); // Return eid
        ReturnCode::Success
    }

    fn run_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Run enclave");

        let eid = ctx.get(Register::X10);
        if eid >= ENCL_MAX || self.enclaves[eid].state != EnclaveState::Fresh {
            return ReturnCode::EnclaveNotFresh;
        }

        Self::context_switch_to_enclave(mctx, ctx, &mut self.enclaves[eid]);
        ReturnCode::Success
    }

    fn destroy_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Destroy enclave");
        let eid = ctx.get(Register::X10);
        if eid >= ENCL_MAX
            || self.enclaves[eid].state == EnclaveState::Running
            || self.enclaves[eid].state == EnclaveState::Invalid
        {
            return ReturnCode::EnclaveNotDestroyable;
        }

        // TODO: Clear data in the enclave pages
        self.enclaves[eid].state = EnclaveState::Invalid;

        // Clear enclave PMPs
        let pmp_id = MODULE_OFFSET + eid * 2;
        mctx.pmp.set_inactive(pmp_id, 0);
        mctx.pmp.set_inactive(pmp_id + 1, 0);
        unsafe {
            write_pmp(&mctx.pmp).flush();
        }

        ReturnCode::Success
    }

    fn resume_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Resume enclave");

        let eid = ctx.get(Register::X10);
        if eid >= ENCL_MAX || self.enclaves[eid].state != EnclaveState::Stopped {
            return ReturnCode::EnclaveNotResumable;
        }

        Self::context_switch_to_enclave(mctx, ctx, &mut self.enclaves[eid]);
        ReturnCode::Success
    }

    fn random(&mut self, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Random");
        ctx.set(Register::X11, self.nonce);
        self.nonce += 1;
        ReturnCode::Success
    }

    fn stop_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Stop enclave");
        let stop_reason = match ctx.trap_info.get_cause() {
            MCause::MachineTimerInt | MCause::MachineSoftInt => StoppedReason::Interrupt,
            _ => StoppedReason::from(ctx.get(Register::X10)),
        };

        let enclave = self.get_active_enclave(ctx).unwrap();
        Self::context_switch_to_host(mctx, ctx, enclave);

        match stop_reason {
            StoppedReason::Interrupt => ReturnCode::EnclaveInterrupted,
            StoppedReason::EdgeCallHost => ReturnCode::EnclaveEdgeCallHost,
            _ => ReturnCode::EnclaveUnknownError,
        }
    }

    fn exit_enclave(&mut self, mctx: &mut MiralisContext, ctx: &mut VirtContext) -> ReturnCode {
        logger::debug!("Keystone: Exit enclave");
        let enclave = self.get_active_enclave(ctx).unwrap();
        let exit_code = ctx.get(Register::X10);
        Self::context_switch_to_host(mctx, ctx, enclave);

        ctx.set(Register::X11, exit_code);
        ReturnCode::Success
    }
}

/// To check how ecalls are handled, see https://github.com/riscv-software-src/opensbi/blob/2ffa0a153d804910c20b82974bfe2dedcf35a777/lib/sbi/sbi_ecall.c#L98
impl Module for KeystonePolicy {
    const NAME: &'static str = "Keystone Policy";

    fn init() -> Self {
        Self::default()
    }

    fn ecall_from_payload(
        &mut self,
        mctx: &mut MiralisContext,
        ctx: &mut VirtContext,
    ) -> ModuleAction {
        let eid = ctx.get(Register::X17);
        let fid = ctx.get(Register::X16);

        if eid != sbi::KEYSTONE_EID {
            return ModuleAction::Ignore;
        }

        let enclave_running = self.get_active_enclave(ctx).is_some();
        let return_code: ReturnCode = match (enclave_running, fid) {
            (false, sbi::CREATE_ENCLAVE_FID) => self.create_enclave(mctx, ctx),
            (false, sbi::DESTROY_ENCLAVE_FID) => self.destroy_enclave(mctx, ctx),
            (false, sbi::RUN_ENCLAVE_FID) => self.run_enclave(mctx, ctx),
            (false, sbi::RESUME_ENCLAVE_FID) => self.resume_enclave(mctx, ctx),
            (true, sbi::RANDOM_FID) => self.random(ctx),
            (true, sbi::STOP_ENCLAVE_FID) => self.stop_enclave(mctx, ctx),
            (true, sbi::EXIT_ENCLAVE_FID) => self.exit_enclave(mctx, ctx),
            _ => ReturnCode::NotImplemented,
        };

        ctx.set(Register::X10, return_code as usize);
        ctx.pc += 4;

        ModuleAction::Overwrite
    }

    fn switch_from_payload_to_firmware(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // If the enclave is currently active, we lock it before transfering control to the
        // firmware
        if let Some(enclave) = self.get_active_enclave(ctx) {
            Self::lock_enclave(mctx, enclave);
        }
    }

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        // If the enclave is currently active, switching to the payload actually switches to the
        // enclave. We need to un-lock it before jumping back.
        if let Some(enclave) = self.get_active_enclave(ctx) {
            Self::unlock_enclave(mctx, enclave);
        }
    }

    const NUMBER_PMPS: usize = ENCL_MAX * 2; // Each enclave uses 2 PMPs because of TOR addressing
}
