//! Benchmark Modules
//!
//! This modules holds the different values for the benchmarks

mod counter;
mod default;
mod empty;

use config_select::select_env;

use crate::benchmark::default::IntervalCounter;
use crate::virt::VirtContext;

pub type Benchmark = select_env!["MIRALIS_BENCHMARK_TYPE":
    "default"      => default::DefaultBenchmark
    "counter"      => counter::CounterBenchmark
    _ => empty::EmptyBenchmark
];

pub trait BenchmarkModule {
    fn init() -> Self;
    fn name() -> &'static str;

    fn start_interval_counters(scope: Scope);
    fn stop_interval_counters(scope: Scope);
    fn increment_counter(counter: Counter);

    fn update_inteval_counter_stats(
        &mut self,
        counter: &IntervalCounter,
        scope: &Scope,
        value: usize,
    );

    /// Print formated string with value of the counters
    fn display_counters();

    /// Read the performance counters into the virtual registers.
    ///
    /// Note: the specific ABI is depends on the benchmark back-end.
    fn read_counters(ctx: &mut VirtContext);

    fn get_counter_value(core_id: usize, counter: Counter) -> usize;
}

pub enum Scope {
    HandleTrap,
    RunVCPU,
}

impl Scope {
    fn base(&self) -> usize {
        match self {
            Self::HandleTrap => 0,
            Self::RunVCPU => 1,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::HandleTrap => "handle_trap",
            Self::RunVCPU => "run_vcpu",
        }
    }
}

/// Benchmark counters.
/// This kind of counter aims to be incremented to count occurences of an event.
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Counter {
    TotalExits = 0,
    FirmwareExits = 1,
    WorldSwitches = 2,
}
