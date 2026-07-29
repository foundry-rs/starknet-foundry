use crate::e2e::common::runner::{setup_package, test_runner};
use forge_runner::DEFAULT_CACHE_DIR;
use std::fs;
use std::path::Path;
use universal_sierra_compiler_api::CASM_CACHE_DIR;

#[test]
fn creates_raw_and_contract_casm_cache_entries() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let casm_cache_dir = temp.path().join(DEFAULT_CACHE_DIR).join(CASM_CACHE_DIR);
    assert!(contains_json_file(&casm_cache_dir.join("raw")));
    assert!(contains_json_file(&casm_cache_dir.join("contract")));
}

fn contains_json_file(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            contains_json_file(&path)
        } else {
            path.extension()
                .is_some_and(|extension| extension == "json")
        }
    })
}
