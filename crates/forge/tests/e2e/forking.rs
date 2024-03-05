use crate::e2e::common::runner::{
    runner, setup_package, setup_package_with_file_patterns, test_runner, BASE_FILE_PATTERNS,
};
use forge::shared_cache::CACHE_DIR;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn without_cache() {
    let temp = setup_package("forking");

    let output = test_runner(&temp)
        .args(["forking::tests::test_fork_simple"])
        .assert()
        .code(0);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from forking package
        Running 4 test(s) from src/
        [PASS] forking::tests::test_fork_simple [..]
        [PASS] forking::tests::test_fork_simple_number_hex [..]
        [PASS] forking::tests::test_fork_simple_hash_hex [..]
        [PASS] forking::tests::test_fork_simple_hash_number [..]
        Tests: 4 passed, 0 failed, 0 skipped, 0 ignored, 1 filtered out
        "},
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

    let output = test_runner(&temp)
        .args(["--exact", "forking::tests::test_fork_simple"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [FAIL] forking::tests::test_fork_simple

        Failure data:
            0x42616c616e63652073686f756c642062652032 ('Balance should be 2')

        Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 4 filtered out

        Failures:
            forking::tests::test_fork_simple
        "},
    );
}

#[test]
fn with_clean_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[&format!("{CACHE_DIR}/*.json")]].concat(),
    );

    runner(&temp).arg("clean-cache").assert().code(0);

    let output = test_runner(&temp)
        .args(["--exact", "forking::tests::test_fork_simple"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::test_fork_simple [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 4 filtered out
        "},
    );
}

#[test]
fn printing_latest_block_number() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[&format!("{CACHE_DIR}/*.json")]].concat(),
    );

    let output = test_runner(&temp)
        .args(["--exact", "forking::tests::print_block_number_when_latest"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::print_block_number_when_latest [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 4 filtered out

        Latest block number = [..] for url = http://188.34.188.184:9545/rpc/v0_6
        "},
    );
}
