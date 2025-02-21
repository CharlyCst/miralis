//! Benchmark Modules
//!
//! This modules holds the different values for the benchmarks

mod counter;
mod counter_per_mcause;
mod default;
mod empty;

use config_select::select_env;

use crate::virt::VirtContext;

pub type Benchmark = select_env!["MIRALIS_BENCHMARK_TYPE":
    "default"      => default::DefaultBenchmark
    "counter"      => counter::CounterBenchmark
    "counter_per_mcause" => counter_per_mcause::CounterPerMcauseBenchmark
    _ => empty::EmptyBenchmark
];

pub trait BenchmarkModule {
    fn init() -> Self;
    fn name() -> &'static str;

    fn start_interval_counters(_scope: Scope) {}
    fn stop_interval_counters(_scope: Scope) {}
    fn increment_counter(_ctx: &mut VirtContext, _counter: Counter) {}

    /// Print formated string with value of the counters
    fn display_counters() {}

    /// Read the performance counters into the virtual registers.
    ///
    /// Note: the specific ABI is depends on the benchmark back-end.
    fn read_counters(_ctx: &mut VirtContext) {}
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
