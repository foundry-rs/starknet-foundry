use super::common::runner::{setup_package, test_runner};
use forge_runner::{DEFAULT_CACHE_DIR, USC_CACHE_DIR};
use std::fs;
use std::path::{Path, PathBuf};
use universal_sierra_compiler_api::supports_cache_dir;

#[test]
fn creates_usc_casm_cache_entries() {
    if !supports_cache_dir().expect("failed to check universal-sierra-compiler version") {
        return;
    }

    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let usc_cache_dir = temp.path().join(DEFAULT_CACHE_DIR).join(USC_CACHE_DIR);
    assert!(
        !json_files(&usc_cache_dir).is_empty(),
        "USC cache should contain at least one JSON entry under {}",
        usc_cache_dir.display()
    );
}

fn json_files(path: &Path) -> Vec<PathBuf> {
    let mut files = vec![];
    collect_json_files(path, &mut files);
    files.sort();
    files
}

fn collect_json_files(path: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    for entry in entries {
        let entry = entry
            .unwrap_or_else(|error| panic!("failed to read entry in {}: {error}", path.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_json_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
}
