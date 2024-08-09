use std::collections::HashMap;

/// Parse a benchmark file in order to get a map from tags to list of usize values.
pub fn parse_content(
    content: Vec<String>,
    map_type_tag_values: &mut HashMap<String, HashMap<String, Vec<usize>>>,
) {
    // keep only benchark logs
    content
        .iter()
        .filter(|s| s.contains("benchmark"))
        // remove separators "||" and collect tuples (bench_type, tag, value)
        .map(|line| {
            let mut split = line.split("||").map(|s| s.trim()).skip(1);
            (
                split
                    .next()
                    .expect("Wrong file format: no benchmark type")
                    .to_string(),
                split.next().expect("Wrong file format: no tag").to_string(),
                split
                    .next()
                    .expect("Wrong file format: no value")
                    .parse::<usize>(),
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
        })
}

pub fn compute_statistics(
    map_type_tag_values: &HashMap<String, HashMap<String, Vec<usize>>>,
    iteration: usize,
) {
    if map_type_tag_values.is_empty() {
        println!("Nothing has been benchmarked !")
    } else {
        println!("\n================= BENCHMARK =================");
    }
    for (key, map) in map_type_tag_values {
        println!("Benchmark for {}:", key);
        println!("--------------------");

        for (tag, values) in map {
            if values.len() == 1 {
                println!("{:.<25}     : {:>12}", tag, values.iter().max().unwrap());
            } else {
                println!("{:.<25}  Min: {:>12}", tag, values.iter().min().unwrap());
                println!("{:.<25}  Max: {:>12}", tag, values.iter().max().unwrap());
                if iteration == 1 {
                    println!("{:.<25}  Sum: {:>12}", tag, values.iter().sum::<usize>());
                } else {
                    println!(
                        "{:.<21} Avg. sum: {:>12}",
                        tag,
                        values.iter().sum::<usize>() / iteration
                    );
                }
                println!(
                    "{:.<25} Mean: {:12}",
                    tag,
                    values.iter().sum::<usize>() / values.len()
                );
                println!();
            }
        }

        println!();
    }
}
