//! Benchmark Modules
//!
//! This modules holds the different values for the benchmarks

mod boot;
mod counter;
mod counter_per_mcause;
mod default;
mod empty;

use config_select::select_env;
use miralis_core::sbi_codes::{
    is_i_fence_request, is_ipi_request, is_timer_request, is_vma_request,
};

use crate::arch::{MCause, Register};
use crate::virt::traits::RegisterContextGetter;
use crate::virt::{ExecutionMode, VirtContext};

pub type Benchmark = select_env!["MIRALIS_BENCHMARK_TYPE":
    "default"      => default::DefaultBenchmark
    "counter"      => counter::CounterBenchmark
    "counter_per_mcause" => counter_per_mcause::CounterPerMcauseBenchmark
    "boot" => boot::BootBenchmark
    _ => empty::EmptyBenchmark
];

pub trait BenchmarkModule {
    fn init() -> Self;
    fn name() -> &'static str;

    fn start_interval_counters(_scope: Scope) {}
    fn stop_interval_counters(_scope: Scope) {}
    fn increment_counter(
        _ctx: &mut VirtContext,
        _from_exec_mode: ExecutionMode,
        _to_exec_mode: ExecutionMode,
    ) {
    }

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

const NUMBER_CATEGORIES: usize = 7;
pub enum ExceptionCategory {
    NotOffloaded = 0,
    ReadTime = 1,
    SetTimer = 2,
    MisalignedOp = 3,
    IPI = 4,
    RemoteFence = 5,
    FirmwareTrap = 6,
}

pub fn get_exception_category(
    ctx: &mut VirtContext,
    from_exec_mode: ExecutionMode,
    to_exec_mode: ExecutionMode,
) -> Option<ExceptionCategory> {
    match (from_exec_mode, to_exec_mode) {
        (ExecutionMode::Payload, ExecutionMode::Firmware) => Some(ExceptionCategory::NotOffloaded),
        (ExecutionMode::Firmware, ExecutionMode::Firmware) => Some(ExceptionCategory::FirmwareTrap),
        (ExecutionMode::Payload, ExecutionMode::Payload) => {
            match MCause::try_from(ctx.trap_info.mcause).unwrap() {
                MCause::StoreAddrMisaligned | MCause::LoadAddrMisaligned => {
                    Some(ExceptionCategory::MisalignedOp)
                }
                MCause::IllegalInstr => Some(ExceptionCategory::ReadTime),
                MCause::EcallFromSMode
                    if is_timer_request(ctx.get(Register::X16), ctx.get(Register::X17)) =>
                {
                    Some(ExceptionCategory::SetTimer)
                }
                MCause::EcallFromSMode
                    if is_ipi_request(ctx.get(Register::X16), ctx.get(Register::X17)) =>
                {
                    Some(ExceptionCategory::IPI)
                }
                MCause::EcallFromSMode
                    if is_i_fence_request(ctx.get(Register::X16), ctx.get(Register::X17)) =>
                {
                    Some(ExceptionCategory::RemoteFence)
                }
                MCause::EcallFromSMode
                    if is_vma_request(ctx.get(Register::X16), ctx.get(Register::X17)) =>
                {
                    Some(ExceptionCategory::RemoteFence)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
