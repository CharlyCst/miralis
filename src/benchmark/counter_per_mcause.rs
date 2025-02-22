/// This benchmark is a bit particular and is use for debugging only. It counts the number of firmware traps and world switches per mcause (illegal instruction, breakpoint,....)
/// This benchmark MUST NOT be used in to measure Miralis and is for DEBUGGING ONLY. When it receives a measurement ecall from the payload, it will print the statistics on the UART only.
/// The reason for this is that we use it only for debugging and we currently don't need to measure this. If this is the case, the benchmark needs to be improved
use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::{Arch, Architecture, Csr, MCause};
use crate::benchmark::BenchmarkModule;
use crate::config::PLATFORM_NB_HARTS;
use crate::virt::{ExecutionMode, VirtContext};

// We don't need to add a padding to avoid false sharing. The size of the struct is a multiplier of the cache line
#[repr(C, align(64))]
#[derive(Debug)]
struct PaddedCounter {
    counter: [AtomicU64; 24],
}

// NOTE: Clippy is triggering a warning here, but it's fine as we use the const only for array
// initialization.
#[allow(clippy::declare_interior_mutable_const)]
const ZEROED_COUNTER: PaddedCounter = PaddedCounter {
    counter: [const { AtomicU64::new(0) }; 24],
};

static NB_WORLD_SWITCHES: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];
static NB_FIRMWARE_EXIT: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];

/// A simple and efficient benchmark module based on atomic counters. It tracks the number of exits per mcause code
///
/// This benchmark is used ONLY for manual debug and helps us to understand how the system behaves
pub struct CounterPerMcauseBenchmark {}

fn raw_cause_to_entry(cause: usize) -> usize {
    mcause_to_entry(MCause::try_from(cause).unwrap())
}

// !! This function doesn't map MCause with the MCause hardware number
// It is used to map the cause to the linear buffer
fn mcause_to_entry(mcause: MCause) -> usize {
    match mcause {
        MCause::InstrAddrMisaligned => 0,
        MCause::InstrAccessFault => 1,
        MCause::IllegalInstr => 2,
        MCause::Breakpoint => 3,
        MCause::LoadAddrMisaligned => 4,
        MCause::LoadAccessFault => 5,
        MCause::StoreAddrMisaligned => 6,
        MCause::StoreAccessFault => 7,
        MCause::EcallFromUMode => 8,
        MCause::EcallFromSMode => 9,
        MCause::EcallFromMMode => 10,
        MCause::InstrPageFault => 11,
        MCause::LoadPageFault => 12,
        MCause::StorePageFault => 13,
        MCause::UserSoftInt => 14,
        MCause::SupervisorSoftInt => 15,
        MCause::MachineSoftInt => 16,
        MCause::UserTimerInt => 17,
        MCause::SupervisorTimerInt => 18,
        MCause::MachineTimerInt => 19,
        MCause::UserExternalInt => 20,
        MCause::SupervisorExternalInt => 21,
        MCause::MachineExternalInt => 22,
        _ => 23, // Unknown exceptions and interrupts
    }
}

macro_rules! log_mcause {
    ($mcause:expr) => {{
        let cause_offset = mcause_to_entry($mcause);
        let hart_id: usize = hard_id();
        log::info!(
            "[{:?} : {:?}] Mcause: {:?}",
            NB_FIRMWARE_EXIT[hart_id].counter[cause_offset],
            NB_WORLD_SWITCHES[hart_id].counter[cause_offset],
            $mcause,
        );
    }};
}

impl BenchmarkModule for CounterPerMcauseBenchmark {
    fn init() -> Self {
        CounterPerMcauseBenchmark {}
    }

    fn name() -> &'static str {
        "Counter per code benchmark"
    }

    fn increment_counter(
        ctx: &mut VirtContext,
        from_exec_mode: ExecutionMode,
        to_exec_mode: ExecutionMode,
    ) {
        let hart_id: usize = hard_id();
        let mcause_offset: usize = raw_cause_to_entry(ctx.trap_info.mcause);

        if from_exec_mode == ExecutionMode::Payload && to_exec_mode == ExecutionMode::Firmware {
            NB_WORLD_SWITCHES[hart_id].counter[mcause_offset].fetch_add(1, Ordering::Relaxed);
        } else if from_exec_mode == ExecutionMode::Firmware
            && to_exec_mode == ExecutionMode::Firmware
        {
            NB_FIRMWARE_EXIT[hart_id].counter[mcause_offset].fetch_add(1, Ordering::Relaxed);
        }
    }

    fn read_counters(_ctx: &mut VirtContext) {
        // For the moment we simply display the counters in Miralis, we use this benchmark for debugging only
        Self::display_counters();

        // Reset values
        for i in 0..24 {
            NB_FIRMWARE_EXIT[hard_id()].counter[i].store(0, Ordering::Relaxed);
            NB_WORLD_SWITCHES[hard_id()].counter[i].store(0, Ordering::Relaxed);
        }
    }

    fn display_counters() {
        log_mcause!(MCause::InstrAddrMisaligned);
        log_mcause!(MCause::InstrAccessFault);
        log_mcause!(MCause::IllegalInstr);
        log_mcause!(MCause::Breakpoint);
        log_mcause!(MCause::LoadAddrMisaligned);
        log_mcause!(MCause::LoadAccessFault);
        log_mcause!(MCause::StoreAddrMisaligned);
        log_mcause!(MCause::StoreAccessFault);
        log_mcause!(MCause::EcallFromUMode);
        log_mcause!(MCause::EcallFromSMode);
        log_mcause!(MCause::EcallFromMMode);
        log_mcause!(MCause::InstrPageFault);
        log_mcause!(MCause::LoadPageFault);
        log_mcause!(MCause::StorePageFault);
        log_mcause!(MCause::UserSoftInt);
        log_mcause!(MCause::SupervisorSoftInt);
        log_mcause!(MCause::MachineSoftInt);
        log_mcause!(MCause::UserTimerInt);
        log_mcause!(MCause::SupervisorTimerInt);
        log_mcause!(MCause::MachineTimerInt);
        log_mcause!(MCause::UserExternalInt);
        log_mcause!(MCause::SupervisorExternalInt);
        log_mcause!(MCause::MachineExternalInt);
    }
}

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Return the current hart id
fn hard_id() -> usize {
    Arch::read_csr(Csr::Mhartid)
}
