// —————————————————————————————— Entry Point ——————————————————————————————— //

use std::path::Path;
use std::{env, fs};

use benchmark::{compute_statistics, parse_content};

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = match args.get(1) {
        Some(s) => Path::new(s),
        None => {
            println!("missing argument \'file_name\'");
            return;
        }
    };

    if !path.exists() {
        println!("File {} doesn't exist.", path.display());
        return;
    }

    let content = read_file_content(path);
    let map_type_tag_values = parse_content(content);

    compute_statistics(map_type_tag_values);
}

fn read_file_content(file_path: &Path) -> Vec<String> {
    fs::read_to_string(file_path)
        .expect("Error while trying to read file.")
        .lines()
        .map(String::from)
        .collect()
}
