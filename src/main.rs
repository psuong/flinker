use regex::Regex;
use std::{
    env::args,
    fs::{hard_link, read_to_string},
    path::Path,
};
use yaml_rust::YamlLoader;

fn main() {
    // The first argument is the path to the yaml file.
    let args: Vec<String> = args().collect();

    let mut iter = args.iter();
    iter.next();

    match iter.next() {
        None => println!("Failed"),
        Some(yaml_path) => {
            let yaml_regex = Regex::new(r".(\byml\b)|(\byaml\b)").unwrap();

            if yaml_regex.is_match(yaml_path) {
                process_yaml_path(yaml_path);
            }
        }
    }
}

fn process_yaml_path(yaml_path: &String) {
    let path = Path::new(yaml_path);
    if !path.exists() {
        return;
    }

    let yaml_contents = read_to_string(path);

    match yaml_contents {
        Ok(v) => println!("parsed"),
        Err(e) => println!("error parsing"),
    }
}
