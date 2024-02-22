use glob::glob;
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

pub fn read_cache(file_pattern: &str) -> Map<String, Value> {
    let mut cache_files = glob(file_pattern).unwrap().filter_map(Result::ok);
    let cache_file = cache_files.next().expect("Cache file not found");

    assert!(
        cache_files.next().is_none(),
        "Multiple matching cache files found"
    );

    let cache_content = fs::read_to_string(cache_file).expect("Could not read cache");
    let parsed_cache_content: Value =
        serde_json::from_str(&cache_content).expect("Could not parse cache");
    parsed_cache_content
        .as_object()
        .expect("Parsed cache is not an object")
        .clone()
}

pub fn purge_cache(directory: &str) {
    fs::remove_dir_all(PathBuf::from_str(directory).expect("Could not parse cache path"))
        .expect("Could not remove cache directory");
}
