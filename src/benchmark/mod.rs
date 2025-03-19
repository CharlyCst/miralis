//! Benchmark Modules
//!
//! This modules holds the different values for the benchmarks

mod boot;
mod counter;
mod counter_per_mcause;
mod empty;

use config_select::select_env;
use miralis_core::sbi_codes::{
    is_i_fence_request, is_ipi_request, is_timer_request, is_vma_request,
};

use crate::arch::{MCause, Register};
use crate::benchmark::ExceptionCategory::{
    FirmwareTrap, MisalignedOp, NotOffloaded, PageFault, ReadTime, RemoteFence, SetTimer, IPI,
};
use crate::virt::traits::RegisterContextGetter;
use crate::virt::{ExecutionMode, VirtContext};

pub type Benchmark = select_env!["MIRALIS_BENCHMARK_TYPE":
    "counter"      => counter::CounterBenchmark
    "counter_per_mcause" => counter_per_mcause::CounterPerMcauseBenchmark
    "boot" => boot::BootBenchmark
    _ => empty::EmptyBenchmark
];

pub trait BenchmarkModule {
    fn init() -> Self;
    fn name() -> &'static str;

    fn increment_counter(
        _ctx: &mut VirtContext,
        _from_exec_mode: ExecutionMode,
        _to_exec_mode: ExecutionMode,
    ) {
    }

    /// Read the performance counters into the virtual registers.
    ///
    /// Note: the specific ABI is depends on the benchmark back-end.
    fn read_counters(_ctx: &mut VirtContext) {}
}

const NUMBER_CATEGORIES: usize = 8;
pub enum ExceptionCategory {
    NotOffloaded = 0,
    ReadTime = 1,
    SetTimer = 2,
    MisalignedOp = 3,
    IPI = 4,
    RemoteFence = 5,
    FirmwareTrap = 6,
    PageFault = 7,
}

impl TryFrom<usize> for ExceptionCategory {
    type Error = ();

    fn try_from(exception_category: usize) -> Result<Self, Self::Error> {
        match exception_category {
            0 => Ok(NotOffloaded),
            1 => Ok(ReadTime),
            2 => Ok(SetTimer),
            3 => Ok(MisalignedOp),
            4 => Ok(IPI),
            5 => Ok(RemoteFence),
            6 => Ok(FirmwareTrap),
            7 => Ok(PageFault),
            _ => Err(()),
        }
    }
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
                MCause::LoadPageFault | MCause::StorePageFault | MCause::InstrPageFault => {
                    Some(ExceptionCategory::PageFault)
                }

                _ => None,
            }
        }
        _ => None,
    }
}
