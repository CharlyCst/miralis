use core::sync::atomic::{AtomicUsize, Ordering};

use crate::arch::{Arch, Architecture, Csr};
use crate::benchmark::{get_exception_category, NUMBER_CATEGORIES};
use crate::config::MODULES;
use crate::policy::offload::OFFLOAD_POLICY_NAME;
use crate::virt::{ExecutionMode, VirtContext};
use crate::BenchmarkModule;

const NUMBER_SECONDS: usize = 15;

const CYCLES_PER_INTERVALL: usize = 2_000_000;

const CSV_HEADER: &str =
    "no-offload, read-time, set-timer, misaligned-op, ipi, remote-fence, firmware-trap";

static BUCKETS: [AtomicUsize; NUMBER_CATEGORIES * NUMBER_SECONDS] =
    [const { AtomicUsize::new(0) }; NUMBER_CATEGORIES * NUMBER_SECONDS];

/// This benchmark is particular. It separates all the exceptions it receives in "Categories" and halts execution after some time.
/// Upon halting the execution it gives an indication over times of what kind of traps arrived when.
/// This allows us to understand the behaviors during the boot of the linux kernel.
/// This benchmark must be used with the offload policy AND IS FOR EXPERIMENTS ONLY
pub struct BootBenchmark {}

impl BenchmarkModule for BootBenchmark {
    fn init() -> Self {
        if !MODULES.contains(&OFFLOAD_POLICY_NAME) {
            panic!("This benchmark must be used with the offload policy")
        }

        BootBenchmark {}
    }

    fn name() -> &'static str {
        "Boot Benchmark"
    }

    fn increment_counter(
        ctx: &mut VirtContext,
        from_exec_mode: ExecutionMode,
        to_exec_mode: ExecutionMode,
    ) {
        if let Some(exception_offset) = get_exception_category(ctx, from_exec_mode, to_exec_mode) {
            let current_time_bin = Arch::read_csr(Csr::Time) / CYCLES_PER_INTERVALL;

            if Self::is_done(current_time_bin) {
                Self::display_benchmark(ctx.hart_id);
            }

            BUCKETS[current_time_bin * NUMBER_CATEGORIES + exception_offset as usize]
                .fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl BootBenchmark {
    fn is_done(current_time_bin: usize) -> bool {
        current_time_bin >= NUMBER_SECONDS
    }

    fn display_benchmark(hart_id: usize) {
        if hart_id != 0 {
            loop {
                Arch::wfi();
            }
        }

        log::info!("{}", CSV_HEADER);
        for i in 0..NUMBER_SECONDS {
            assert!(i * NUMBER_CATEGORIES + 1 < NUMBER_CATEGORIES * NUMBER_SECONDS);
            log::info!(
                "{},{},{},{},{},{},{}",
                BUCKETS[i * NUMBER_CATEGORIES].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 1].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 2].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 3].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 4].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 5].load(Ordering::SeqCst),
                BUCKETS[i * NUMBER_CATEGORIES + 6].load(Ordering::SeqCst)
            );
        }

        loop {
            Arch::wfi();
        }
    }
}
