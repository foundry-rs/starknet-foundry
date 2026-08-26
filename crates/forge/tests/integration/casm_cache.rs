use crate::utils::running_tests::{RunTestCaseOptions, run_test_case_with_options};
use crate::utils::{runner::assert_passed, test_case};
use camino::Utf8PathBuf;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use shared::cache::usc_cache_dir;
use tempfile::tempdir;
use walkdir::WalkDir;

#[test]
fn uses_custom_cache_dir() {
    let test = test_case!(indoc!(
        r"#[test]
        fn simple() {
            assert(2 == 2, '2 == 2');
        }
    "
    ));
    let temp = tempdir().unwrap();
    let custom_cache_dir = Utf8PathBuf::from_path_buf(temp.path().join("custom_cache")).unwrap();

    let result = run_test_case_with_options(
        &test,
        ForgeTrackedResource::CairoSteps,
        RunTestCaseOptions {
            cache_dir: custom_cache_dir.clone(),
        },
    );

    assert_passed(&result);
    let usc_cache_dir = usc_cache_dir(&custom_cache_dir);
    assert!(
        WalkDir::new(&usc_cache_dir).into_iter().any(|entry| {
            entry.is_ok_and(|entry| {
                entry
                    .path()
                    .file_name()
                    .is_some_and(|name| name == "casm.json")
            })
        }),
        "USC cache should contain a JSON entry under {usc_cache_dir}"
    );
}
