use crate::benchmark::BenchmarkModule;

pub struct EmptyBenchmark {}

impl BenchmarkModule for EmptyBenchmark {
    fn init() -> Self {
        EmptyBenchmark {}
    }

    fn name() -> &'static str {
        "Empty Benchmark"
    }
}
