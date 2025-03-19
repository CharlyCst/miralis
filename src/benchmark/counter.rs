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
    page_faults: AtomicU64,
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
    page_faults: const { AtomicU64::new(0) },
    _padding: [0; 64 - 7 * size_of::<AtomicU64>()],
};

static COUNTERS: [PaddedCounter; PLATFORM_NB_HARTS] = [ZEROED_COUNTER; PLATFORM_NB_HARTS];

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
                    .remote_fence_request
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::NotOffloaded) => {
                COUNTERS[ctx.hart_id]
                    .world_switches
                    .fetch_add(1, Ordering::Relaxed);
            }
            Some(ExceptionCategory::PageFault) => {
                COUNTERS[ctx.hart_id]
                    .page_faults
                    .fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn read_counters(ctx: &mut VirtContext) {
        let hart_to_read = ctx.get(Register::X10);
        let exception_category = ExceptionCategory::try_from(ctx.get(Register::X11)).unwrap();

        let measure = match exception_category {
            ExceptionCategory::NotOffloaded => {
                COUNTERS[hart_to_read].world_switches.load(Ordering::SeqCst)
            }
            ExceptionCategory::ReadTime => COUNTERS[hart_to_read].timer_read.load(Ordering::SeqCst),
            ExceptionCategory::SetTimer => {
                COUNTERS[hart_to_read].timer_request.load(Ordering::SeqCst)
            }
            ExceptionCategory::MisalignedOp => {
                COUNTERS[hart_to_read].misaligned_op.load(Ordering::SeqCst)
            }
            ExceptionCategory::IPI => COUNTERS[hart_to_read].ipi_request.load(Ordering::SeqCst),
            ExceptionCategory::RemoteFence => COUNTERS[hart_to_read]
                .remote_fence_request
                .load(Ordering::SeqCst),
            ExceptionCategory::FirmwareTrap => {
                COUNTERS[hart_to_read].firmware_traps.load(Ordering::SeqCst)
            }
            ExceptionCategory::PageFault => {
                COUNTERS[hart_to_read].page_faults.load(Ordering::SeqCst)
            }
        };

        ctx.set(Register::X10, measure as usize);
    }
}
