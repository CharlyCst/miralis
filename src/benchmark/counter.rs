use core::sync::atomic::{AtomicU64, Ordering};

use crate::arch::Register;
use crate::benchmark::{get_exception_category, BenchmarkModule, ExceptionCategory};
use crate::config::PLATFORM_NB_HARTS;
use crate::virt::traits::*;
use crate::virt::{ExecutionMode, VirtContext};

// We use this structure to avoid false sharing in the benchmark.
// The typical size of a cache line is 64 bytes
#[repr(C, align(64))]
#[derive(Debug, Default)]
struct PaddedCounter {
    firmware_traps: AtomicU64,
    world_switches: AtomicU64,
    misaligned_op: AtomicU64,
    timer_read: AtomicU64,
    timer_request: AtomicU64,
    ipi_request: AtomicU64,
    remote_fence_request: AtomicU64,
    _padding: [u8; 64 - 7 * size_of::<AtomicU64>()],
}

// NOTE: Clippy is triggering a warning here but it's fine as we use the const only for array
// initialization.
#[allow(clippy::declare_interior_mutable_const)]
const ZEROED_COUNTER: PaddedCounter = PaddedCounter {
    firmware_traps: const { AtomicU64::new(0) },
    world_switches: const { AtomicU64::new(0) },
    misaligned_op: const { AtomicU64::new(0) },
    timer_read: const { AtomicU64::new(0) },
    timer_request: const { AtomicU64::new(0) },
    ipi_request: const { AtomicU64::new(0) },
    remote_fence_request: const { AtomicU64::new(0) },
    _padding: [0; 64 - 7 * size_of::<AtomicU64>()],
};

static COUNTERS: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];

const SINGLE_CORE_BENCHMARK: usize = 0;
const ALL_CORES_BENCHMARK: usize = 1;

/// A simple and efficient benchmark module based on atomic counters.
///
/// This benchmark module explicitly avoid computing any advanced statistics (e.g. standard
/// deviation) to keep the code simple and efficient.
pub struct CounterBenchmark {}

impl BenchmarkModule for CounterBenchmark {
    fn init() -> Self {
        CounterBenchmark {}
    }

    fn name() -> &'static str {
        "Counter benchmark"
    }

    fn increment_counter(
        ctx: &mut VirtContext,
        from_exec_mode: ExecutionMode,
        to_exec_mode: ExecutionMode,
    ) {
        match get_exception_category(ctx, from_exec_mode, to_exec_mode) {
            Some(ExceptionCategory::FirmwareTrap) => {
                COUNTERS[ctx.hart_id]
                    .firmware_traps
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::ReadTime) => {
                COUNTERS[ctx.hart_id]
                    .timer_read
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::SetTimer) => {
                COUNTERS[ctx.hart_id]
                    .timer_request
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::MisalignedOp) => {
                COUNTERS[ctx.hart_id]
                    .misaligned_op
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::IPI) => {
                COUNTERS[ctx.hart_id]
                    .ipi_request
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::RemoteFence) => {
                COUNTERS[ctx.hart_id]
                    .timer_request
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::NotOffloaded) => {
                COUNTERS[ctx.hart_id]
                    .world_switches
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn read_counters(ctx: &mut VirtContext) {
        let mut nb_firmware_exits: usize = 0;
        let mut nb_world_switch: usize = 0;

        match ctx.get(Register::X10) {
            SINGLE_CORE_BENCHMARK => {
                nb_firmware_exits = get_nb_firmware_exits(ctx.hart_id) as usize;
                nb_world_switch = get_nb_world_switch(ctx.hart_id) as usize;
            }
            ALL_CORES_BENCHMARK => {
                for current_hart in 0..PLATFORM_NB_HARTS {
                    nb_firmware_exits += get_nb_firmware_exits(current_hart) as usize;
                    nb_world_switch += get_nb_world_switch(current_hart) as usize;
                }
            }
            _ => log::error!(
                "Invalid argument for register a0 [0 ==> Core 0 | 1 ==> All cores] {}",
                ctx.get(Register::X10)
            ),
        }

        ctx.set(Register::X10, nb_firmware_exits);
        ctx.set(Register::X11, nb_world_switch);
    }
}

// ———————————————————————————————— Helpers ————————————————————————————————— //

/// Return the number of firmware exits on the given hart
fn get_nb_firmware_exits(hart: usize) -> u64 {
    COUNTERS[hart].firmware_traps.load(Ordering::Relaxed)
}

/// Return the number of world switches on the given hart
fn get_nb_world_switch(hart: usize) -> u64 {
    COUNTERS[hart].world_switches.load(Ordering::Relaxed)
}
