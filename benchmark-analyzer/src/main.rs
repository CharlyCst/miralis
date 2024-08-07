// —————————————————————————————— Entry Point ——————————————————————————————— //

use std::collections::HashMap;
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

    // Map of Benchmark type -> Tag -> values
    let mut map_type_tag_values: HashMap<String, HashMap<String, Vec<usize>>> = HashMap::new();

    if path.is_dir() {
        path.read_dir()
            .unwrap()
            .map(|res| res.map(|e| e.path()).unwrap())
            .filter(|file_path| file_path.is_file())
            .map(|file_path| read_file_content(&file_path))
            .for_each(|c| parse_content(c, &mut map_type_tag_values));

        compute_statistics(&map_type_tag_values, path.read_dir().unwrap().count());
    } else {
        let content = read_file_content(path);
        parse_content(content, &mut map_type_tag_values);

        compute_statistics(&map_type_tag_values, 1);
    }
}

fn read_file_content(file_path: &Path) -> Vec<String> {
    fs::read_to_string(file_path)
        .expect("Error while trying to read file.")
        .lines()
        .map(String::from)
        .collect()
}
