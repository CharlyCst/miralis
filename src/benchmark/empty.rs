use crate::benchmark::default::IntervalCounter;
use crate::benchmark::{BenchmarkModule, Counter, Scope};

pub struct EmptyBenchmark {}

impl BenchmarkModule for EmptyBenchmark {
    fn init() -> Self {
        EmptyBenchmark {}
    }

    fn name() -> &'static str {
        "Empty Benchmark"
    }

    fn start_interval_counters(_scope: Scope) {}

    fn stop_interval_counters(_scope: Scope) {}

    fn increment_counter(_counter: Counter) {}

    fn update_inteval_counter_stats(
        &mut self,
        _counter: &IntervalCounter,
        _scope: &Scope,
        _value: usize,
    ) {
    }

    fn display_counters() {}

    fn get_counter_value(_counter: Counter) -> usize {
        0
    }
}
