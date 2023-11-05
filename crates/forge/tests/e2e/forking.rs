use crate::assert_stdout_contains;
use crate::e2e::common::runner::{
    runner, setup_package, setup_package_with_file_patterns, test_runner, BASE_FILE_PATTERNS,
};
use forge::CACHE_DIR;
use indoc::indoc;

#[test]
fn without_cache() {
    let temp = setup_package("forking");
    let snapbox = test_runner();

    let output = snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::tests::test_fork_simple"])
        .assert()
        .code(0);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::test_fork_simple
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
/// The cache file at `forking/$CACHE_DIR` was modified to have different value stored
/// that this from the real network. We use it to verify that values from cache are actually used.
///
/// The test that passed when using data from network, should fail for fabricated data.
fn with_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[&format!("{CACHE_DIR}/*.json")]].concat(),
    );
    let snapbox = test_runner();

    let output = snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::tests::test_fork_simple"])
        .assert()
        .code(1);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [FAIL] forking::tests::test_fork_simple
        
        Failure data:
            original value: [1480335954842313548834020101284630397133856818], converted to a string: [Balance should be 2]
        
        Tests: 0 passed, 1 failed, 0 skipped

        Failures:
            forking::tests::test_fork_simple
        "#}
    );
}

#[test]
fn with_clean_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[&format!("{CACHE_DIR}/*.json")]].concat(),
    );

    runner()
        .arg("clean-cache")
        .current_dir(&temp)
        .assert()
        .code(0);

    let output = test_runner()
        .current_dir(&temp)
        .args(["--exact", "forking::tests::test_fork_simple"])
        .assert()
        .code(0);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::test_fork_simple
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn printing_latest_block_number() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[&format!("{CACHE_DIR}/*.json")]].concat(),
    );
    let snapbox = test_runner();

    let output = snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::tests::print_block_number_when_latest"])
        .assert()
        .code(0);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::print_block_number_when_latest
        Number of the block used for fork testing = [..]
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}
