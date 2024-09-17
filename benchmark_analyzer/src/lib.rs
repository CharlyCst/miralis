use std::collections::HashMap;

const CSV_SEPARATOR: char = ',';
const SCOPE_SEPARATOR: &str = "::";
const COUNTER_SCOPE: &str = "counters";
const START_TOKEN: &str = "START BENCHMARK";

/// Parse a benchmark file in order to get a map from tags to list of usize values.
pub fn parse_content(
    content: Vec<String>,
    stat_counter_values_map: &mut HashMap<String, HashMap<String, Vec<usize>>>,
) {
    let mut results = content
        .iter()
        .skip_while(|s| !s.contains(START_TOKEN))
        .skip(1);

    // Retrieve statistics names
    let stats: Vec<&str> = results
        .next()
        .expect("Not a benchmark-compatible firmware!")
        .split(CSV_SEPARATOR)
        .skip(1)
        .collect();

    results.for_each(|line| {
        let mut split = line.split(CSV_SEPARATOR).map(|s| s.trim());
        let counter_name = split.next().expect("Missing counter name."); // Counter name

        stats.iter().for_each(|key| {
            stat_counter_values_map
                .entry(key.to_string())
                .or_default()
                .entry(counter_name.to_string())
                .or_default()
                .push(
                    split
                        .next()
                        .expect("Wrong file format: no value")
                        .parse::<usize>()
                        .expect("Wrong file format: value is not an usize"),
                )
        });
    });
}

#[derive(Default, Debug)]
struct CounterStats {
    min: usize,
    max: usize,
    mean: usize,
    avg_sum: usize,
}

/// Compute average of all parameters to have statistics over all runs.
pub fn compute_statistics(stat_counter_values_map: &HashMap<String, HashMap<String, Vec<usize>>>) {
    if stat_counter_values_map.is_empty() {
        println!("Nothing has been benchmarked !");
        return;
    }

    let mut scope_stats_counters: HashMap<String, HashMap<String, CounterStats>> = HashMap::new();

    for (stat, map) in stat_counter_values_map {
        for (counter_names, values) in map {
            let mut split = counter_names.split(SCOPE_SEPARATOR);
            let counter_name = split.next().expect("No counter name!");
            let scope_name = split.next().unwrap_or(COUNTER_SCOPE);
            let a = scope_stats_counters
                .entry(scope_name.to_string())
                .or_default()
                .entry(counter_name.to_string())
                .or_default();

            if stat == "min" {
                a.min = *values.iter().min().unwrap()
            } else if stat == "max" {
                a.max = *values.iter().max().unwrap()
            } else if stat == "sum" {
                a.avg_sum = values.iter().sum::<usize>() / values.len();
            } else if stat == "mean" {
                a.mean = values.iter().sum::<usize>() / values.len();
            }
        }
    }

    print_stats(&scope_stats_counters);
}

/// Print formatted statistics and numbers.
fn print_stats(scope_stats_counters: &HashMap<String, HashMap<String, CounterStats>>) {
    for (scope, map) in scope_stats_counters {
        println!("╔{:─>30}╗", "");
        println!("│{:^30}│", scope);

        for (counter, stats) in map {
            println!("│╔{:─^28}╗│", format!(" {} ", counter));

            if scope == COUNTER_SCOPE {
                println!("││  Count: {:>18} ││", stats.max);
            } else {
                println!("││  Min: {:>20} ││", stats.min);
                println!("││  Max: {:>20} ││", stats.max);
                println!("││  Avg. sum: {:>15} ││", stats.avg_sum);
                println!("││  Mean: {:>19} ││", stats.mean);
            }

            println!("│╚{:─>28}╝│", "");
        }
        println!("╚{:─>30}╝", "");
    }
}
