use super::common::runner::{setup_package, test_runner};
use camino::Utf8PathBuf;
use shared::cache::{DEFAULT_CACHE_DIR, usc_cache_dir};
use walkdir::WalkDir;

#[test]
fn creates_usc_casm_cache_entries() {
    let temp = setup_package("targets/unit_and_integration");

    test_runner(&temp).assert().success();

    let cache_dir = Utf8PathBuf::from_path_buf(temp.path().join(DEFAULT_CACHE_DIR))
        .expect("cache path should be valid UTF-8");
    let usc_cache_dir = usc_cache_dir(&cache_dir);
    assert!(
        WalkDir::new(&usc_cache_dir).into_iter().any(|entry| {
            entry.is_ok_and(|entry| {
                entry
                    .path()
                    .file_name()
                    .is_some_and(|name| name == "casm.json")
            })
        }),
        "USC cache should contain at least one casm.json entry under {usc_cache_dir}"
    );
}
