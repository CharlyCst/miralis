// —————————————————————————————— Entry Point ——————————————————————————————— //

use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let path = Path::new("./bench.out");

    if !path.exists() {
        return;
    }

    println!("\n================= BENCHMARK =================");

    let map_tag_values = parse_file(path);

    for (key, map) in map_tag_values {
        println!("\t   Benchmark for {}:", key);
        for (tag, values) in map {
            println!("{:.<25}  Sum: {:12}", tag, values.iter().sum::<usize>());

            println!(
                "{:.<25} Mean: {:12}",
                tag,
                values.iter().sum::<usize>() / values.len()
            );
        }
        println!();
    }
}

/// Parse a benchmark file in order to get a map from tags to list of usize values.
fn parse_file(file_path: &Path) -> HashMap<String, HashMap<String, Vec<usize>>> {
    let mut map_type_tag_values: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();

    fs::read_to_string(file_path)
        .expect("Error while trying to read file.")
        .lines()
        // keep only benchark logs
        .filter(|s| s.contains("benchmark"))
        // remove separators "||" and collect tuples (bench_type, tag, value)
        .map(|line| {
            let mut split = line.split("||").map(|s| s.trim()).skip(1);
            (
                split.next().unwrap().to_string(),
                split.next().unwrap().to_string(),
                split.next().unwrap().parse::<usize>(),
            )
        })
        // filter incorrect usize transformation
        .filter(|(_, _, value)| value.is_ok())
        // add tuple to the map
        .for_each(|(bench_type, tag, value)| {
            map_type_tag_values
                .entry(bench_type)
                .or_insert(HashMap::new())
                .entry(tag)
                .or_insert(Vec::new())
                .push(value.unwrap())
        });

    map_type_tag_values
}
