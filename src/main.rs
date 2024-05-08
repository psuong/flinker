use dirs::home_dir;
use env_logger::{Builder, Target};
use log::{error, info, LevelFilter};
use regex::Regex;
use std::{
    env::{self, args},
    ffi::OsString,
    fs::{hard_link, read_to_string, remove_file},
    io::{self, Result},
    os::windows::fs::{symlink_dir, symlink_file},
    path::Path,
};
use yaml_rust2::{Yaml, YamlLoader};

fn main() {
    let _ = Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .init();

    let environment_dirs = collect_environment_directories();

    for (key, value) in environment_dirs {
        println!("{}, {}", key, value)
    }
    return;

    // The first argument is the path to the yaml file.
    let args: Vec<String> = args().collect();
    let mut iter = args.iter();
    iter.next();

    match iter.next() {
        None => error!("Path argument not supplied!"),
        Some(yaml_path) => {
            let yaml_regex = Regex::new(r".(\byml\b)|(\byaml\b)").unwrap();

            if yaml_regex.is_match(yaml_path) {
                load_yaml_contents(yaml_path);
            } else {
                error!("{} is not a YAML path!", yaml_path);
            }
        }
    }
}

/// Grabs the YAML's content given a valid path.
///
/// # Arguments
///
/// * `yaml_path` - The relative or absolute path to the YAML file
///
/// # Examples
///
/// ```
/// let abs_path = "/source/example.yml";
/// load_yaml_contents(&abs_path);
/// ```
fn load_yaml_contents(yaml_path: &String) {
    let path = Path::new(yaml_path);
    if !is_file(&path) {
        error!("{} is not a valid path to a YAML file", yaml_path);
        return;
    }

    let yaml_contents = read_to_string(path);

    match yaml_contents {
        Ok(v) => parse_yaml_contents(v),
        Err(e) => error!(
            "Failed to read the contents at: {}. Outputting stack: {}",
            yaml_path, e
        ),
    }
}

/// Parses the YAML file for a specific dictionary structure.
///
/// # Arguments
///
/// * `contents` - The contents of the YAML file
///
/// # Examples
///
/// ```
/// let yaml_contents = "hardlink:
///     - src: a.txt
///     - dst: b.txt
/// ";
///
/// parse_yaml_contents(yaml_contents);
/// ```
fn parse_yaml_contents(contents: String) {
    let result = YamlLoader::load_from_str(&contents).unwrap();
    let symlinker = |src: &Path, dst: &Path| -> Result<()> { symlink_file(src, dst) };

    let hardlinker = |src: &Path, dst: &Path| -> Result<()> { hard_link(src, dst) };

    let dirlink = |src: &Path, dst: &Path| -> Result<()> { symlink_dir(src, dst) };

    for doc in &result {
        execute_file_linker(&doc["symlink"], symlinker);
        execute_file_linker(&doc["hardlink"], hardlinker);
        execute_directory_symlinker(&doc["symlink-dir"], dirlink);
    }
}

/// Executes any function that processes 2 paths and returns an IO Result.
///
/// # Arguments
///
/// * `linker_type` - The linker type defined in the YAML
/// * `linker_function` - The linker function to execute, typically symlinked or hard linked files
///
/// # Examples
///
/// ```
/// execute_file_link(&some_yaml, |src: &Path, dst: &Path| -> io::Result<()> {
///     hard_link(src, dst)
/// });
/// ```
fn execute_file_linker<F>(linker_type: &Yaml, linker_function: F)
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    if !linker_type.is_badvalue() {
        let src = &linker_type[0]["src"];
        let dst = &linker_type[1]["dst"];

        if !&src.is_badvalue() && !&dst.is_badvalue() {
            let src_path = Path::new(src.as_str().unwrap());
            let dst_path = Path::new(dst.as_str().unwrap());

            if is_file(&dst_path) {
                let _ = remove_file(&dst_path);
            }

            if is_file(&src_path) {
                // The paths exist so do a hard link
                match linker_function(src_path, dst_path) {
                    Ok(_) => info!(
                        "Successfully linked from {} -> {}",
                        src.as_str().unwrap(),
                        dst.as_str().unwrap()
                    ),
                    Err(e) => error!(
                        "Failed to link from {} -> {}. \n {}",
                        src.as_str().unwrap(),
                        dst.as_str().unwrap(),
                        e
                    ),
                }
            }
        } else {
            error!("Failed to parse the src and dst values from the YAML!");
        }
    }
}

fn execute_directory_symlinker<F>(linker_type: &Yaml, linker_function: F)
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    if !linker_type.is_badvalue() {
        let src = &linker_type[0]["src"];
        let dst = &linker_type[1]["dst"];

        if !&src.is_badvalue() && !&dst.is_badvalue() {
            let src_path = Path::new(src.as_str().unwrap());

            if src_path.is_dir() {
                let dst_path = Path::new(dst.as_str().unwrap());

                match linker_function(src_path, dst_path) {
                    Ok(_) => info!(
                        "Successfully linked directory: {} -> {}",
                        src.as_str().unwrap(),
                        dst.as_str().unwrap()
                    ),
                    Err(e) => error!(
                        "Failed to link directory from {} -> {} \n {}",
                        src.as_str().unwrap(),
                        dst.as_str().unwrap(),
                        e
                    ),
                }
            }
        }
    }
}

fn collect_environment_directories() -> Vec<(String, String)>{
    let mut filtered_vars: Vec<(String, String)> = env::vars_os()
        .into_iter()
        .filter(|(key, value)| {
            let path = Path::new(value);
            path.exists() && path.is_dir()
        })
        .map(|(key, value)| {
            (key.to_str().unwrap().to_string(), value.to_str().unwrap().to_string()) 
        })
        .collect();

    let home_value = home_dir().unwrap().to_str().unwrap().to_string();
    let home_key = "HOME".to_string();
    // let tuple = (home_key, home_value);
    // filtered_vars.append(tuple);

    filtered_vars.sort_by(|a, b| {
        let (a_key, _) = a;
        let (b_key, _) = b;
        a_key.partial_cmp(b_key).unwrap()
    });

    filtered_vars
}

fn try_read_relative_aliases(path: String) {
    let path_buffer = Path::new(&path);
    info!("Src: {}, Exists: {}", path, path_buffer.exists());
}

/// Checks if the path exists and is a file.
///
/// # Arguments
///
/// * `path` - The relative path to check
///
/// # Examples
///
/// ```
/// let path = Path::new("a.txt");
/// bool is_file = is_file(&path);
/// ```
fn is_file(path: &Path) -> bool {
    path.exists() && path.is_file()
}
