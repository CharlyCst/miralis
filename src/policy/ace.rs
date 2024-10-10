//! The ace security policy. This policy colocates the ACE security monitor (https://github.com/IBM/ACE-RISCV) with Miralis such that the Firmware can be untrusted..

use core::sync::atomic::{AtomicBool, Ordering};
use core::sync::atomic::Ordering::SeqCst;
use crate::ace::core::architecture::control_status_registers::ReadWriteRiscvCsr;
use crate::ace::core::architecture::CSR;
use crate::ace::core::control_data::HardwareHart;
use crate::ace::core::initialization::{ace_setup_this_hart, HARTS_STATES};
use crate::arch::{parse_mpp_return_mode, Arch, Architecture};
use crate::device_tree::divide_memory_region_size;
use crate::host::MiralisContext;
use crate::monitor_switch::{
    address_to_miralis_context, address_to_policy, address_to_virt_context,
    overwrite_hardware_hart_with_virtctx, overwrite_virtctx_with_hardware_hart,
};
use crate::policy::{Policy, PolicyHookResult, PolicyModule};
use crate::virt::VirtContext;
use crate::{ace, handle_trap, main_loop};

extern "C" {
    // Assembly function that is an entry point to the security monitor from the hypervisor or a virtual machine.
    fn enter_from_hypervisor_or_vm_asm() -> !;

    /// To ensure safety, specify all possible valid states that KVM expects to see and prove that security monitor
    /// never returns to KVM with other state. For example, only a subset of exceptions/interrupts can be handled by KVM.
    /// KVM kill the vcpu if it receives unexpected exception because it does not know what to do with it.
    fn exit_to_hypervisor_asm() -> !;

    // Miralis raw trap handler
    fn _raw_trap_handler();
}

/// The ACE policy, which colocates the ACE security monitor with Miralis
pub struct AcePolicy {}

/// This functions transfers the control to the ACE security monitor from Miralis
fn miralis_to_ace_ctx_switch(
    virt_ctx: &mut VirtContext,
    mctx: &mut MiralisContext,
    policy: &mut AcePolicy,
) {
    // Step 0: Get ACE Context
    let hart_id = virt_ctx.hart_id;
    /*assert!(
        hart_id == 0,
        "Implement this code for multihart - don't forget the while !HARTS_STATES.is_completed"
    );*/

    let mut harts = HARTS_STATES
        .get()
        .expect("Bug. Could not set mscratch before initializing memory region for harts states")
        .lock();
    let ace_ctx: &mut HardwareHart = harts
        .get_mut(hart_id)
        .expect("Bug. Incorrectly setup memory region for harts states");

    unsafe {
        HARTS_STATES
            .get()
            .expect(
                "Failure unlock",
            )
            .force_unlock();
    }

    // Step 1: Overwrite Hardware hart with virtcontext
    overwrite_hardware_hart_with_virtctx(ace_ctx, mctx, virt_ctx);
    // Step 1-bis: Set mepc value to pc before jumping
    ace_ctx.hypervisor_hart.hypervisor_hart_state.csrs.mepc = ReadWriteRiscvCsr(virt_ctx.pc);

    ace_ctx.ctx_ptr = (virt_ctx as *const VirtContext) as usize;
    ace_ctx.mctx_ptr = (mctx as *const MiralisContext) as usize;
    ace_ctx.policy_ptr = (policy as *const AcePolicy) as usize;

    // TODO: Is it enough?
    // Step 2: Change mscratch value
    CSR.mscratch.write(ace_ctx as *const _ as usize);

    // Step 3: Change trap handler
    let trap_vector_address = enter_from_hypervisor_or_vm_asm as usize;
    ace_ctx
        .hypervisor_hart_mut()
        .csrs_mut()
        .mtvec
        .write((trap_vector_address >> 2) << 2);

    log::debug!("Firmware -> Payload");

    // Step 4: Jump to the payload - todo: Do we need to apply a response? It seems not to be the case
    unsafe {
        exit_to_hypervisor_asm();
    }
}

pub fn ace_to_miralis_ctx_switch(ace_ctx: &mut HardwareHart) -> ! {
    // Step 0: Get miralis contexts
    let ctx: &mut VirtContext = address_to_virt_context(ace_ctx.ctx_ptr);
    let mctx: &mut MiralisContext = address_to_miralis_context(ace_ctx.mctx_ptr);
    let policy: &mut Policy = address_to_policy(ace_ctx.policy_ptr);

    // Step 1: Fill virt context from hardware hart
    overwrite_virtctx_with_hardware_hart(ctx, mctx, ace_ctx);
    // Todo: restore the stack pointer register here
    ctx.pc = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mepc
        .read();
    ctx.trap_info.mtval = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mtval
        .read();
    ctx.trap_info.mepc = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mepc
        .read();
    ctx.trap_info.mcause = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mcause
        .read();
    ctx.trap_info.mip = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mip
        .read();
    ctx.trap_info.mstatus = ace_ctx
        .hypervisor_hart
        .hypervisor_hart_state
        .csrs
        .mstatus
        .read();

    // restore correct trap handler

    // Restoring the current mode
    ctx.mode = parse_mpp_return_mode(ctx.trap_info.mstatus);

    // Step 2: Change mscratch value
    // Normally here we should not do anything as miralis installs mepc in _run_vcpu

    // Step 3: Change trap handler - install Miralis trap handler
    Arch::install_handler(_raw_trap_handler as usize);

    // Step 4: Jump in the Miralis trap handler - and enter the main loop
    log::debug!("Payload -> Firmware {:?}", ctx.trap_info);
    handle_trap(ctx, mctx, policy);

    main_loop(ctx, mctx, policy);
}
static SETUP_READY: AtomicBool = AtomicBool::new(false);


impl PolicyModule for AcePolicy {
    fn init(mctx: &mut MiralisContext, device_tree_blob_addr: usize) -> Self {
        // INIT ACE
        if mctx.hw.hart == 0 {
            // Step 1: Break forward tree
            match divide_memory_region_size(device_tree_blob_addr) {
                Ok(_) => log::debug!("Splitted the device tree with success"),
                Err(e) => log::error!("Failed to split the device tree {:?}", e),
            }

            // Step 2: Initialise
            match ace::core::initialization::init_security_monitor(device_tree_blob_addr as *const u8) {
                Ok(_) => log::info!("Initialized ACE security monitor."),
                Err(e) => log::error!("Error occurred: {:?}", e),
            }
            SETUP_READY.store(true, Ordering::SeqCst);
        } else {
            while !SETUP_READY.load(Ordering::SeqCst) {
                core::hint::spin_loop();
            }
        }


        // Step 3: Call setup this hard (Todo: Refactor for multicore)
        ace_setup_this_hart(mctx);
        // END INIT ACE

        AcePolicy {}
    }

    fn name() -> &'static str {
        "ACE policy"
    }

    fn ecall_from_firmware(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn ecall_from_payload(
        &mut self,
        _mctx: &mut MiralisContext,
        _ctx: &mut VirtContext,
    ) -> PolicyHookResult {
        PolicyHookResult::Ignore
    }

    fn switch_from_payload_to_firmware(&mut self, _: &mut VirtContext, _: &mut MiralisContext) {}

    fn switch_from_firmware_to_payload(
        &mut self,
        ctx: &mut VirtContext,
        mctx: &mut MiralisContext,
    ) {
        miralis_to_ace_ctx_switch(ctx, mctx, self)
    }

    fn on_interrupt(&mut self, _ctx: &mut VirtContext, _mctx: &mut MiralisContext) {
        todo!("Implement on_interrupt for ace security monitor")
    }

    const NUMBER_PMPS: usize = 2;
}
