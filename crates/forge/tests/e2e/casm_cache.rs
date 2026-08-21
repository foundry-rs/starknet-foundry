use super::common::runner::{setup_package, test_runner};
use forge::run_tests::cache::USC_CACHE_DIR;
use shared::cache::DEFAULT_CACHE_DIR;
use walkdir::WalkDir;

#[test]
fn creates_usc_casm_cache_entries() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let usc_cache_dir = temp.path().join(DEFAULT_CACHE_DIR).join(USC_CACHE_DIR);
    assert!(
        WalkDir::new(&usc_cache_dir).into_iter().any(|entry| {
            entry.is_ok_and(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "json")
            })
        }),
        "USC cache should contain at least one JSON entry under {}",
        usc_cache_dir.display()
    );
}
