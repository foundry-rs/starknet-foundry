use crate::assert_stdout_contains;
use crate::e2e::common::runner::{
    runner, setup_package, setup_package_with_file_patterns, BASE_FILE_PATTERNS,
};
use indoc::indoc;

#[test]
fn without_cache() {
    let temp = setup_package("forking");
    let snapbox = runner();

    let output = snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::test_fork_simple"])
        .assert()
        .code(0);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::test_fork_simple
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
/// The cache file at `forking/.snfoundry_cache/` was modified to have different value stored
/// that this from the real network. We use it to verify that values from cache are actually used.
///
/// The test that passed when using data from network, should fail for fabricated data.
fn with_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[".snfoundry_cache/*.json"]].concat(),
    );
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [FAIL] forking::test_fork_simple
        
        Failure data:
            original value: [1480335954842313548834020101284630397133856818], converted to a string: [Balance should be 2]
        
        Tests: 0 passed, 1 failed, 0 skipped

        Failures:
            forking::test_fork_simple
        "#}
    );
}

#[test]
fn with_clean_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[".snfoundry_cache/*.json"]].concat(),
    );
    let snapbox = runner();

    let output = snapbox
        .current_dir(&temp)
        .arg("--clean-cache")
        .assert()
        .code(0);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::test_fork_simple
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}
