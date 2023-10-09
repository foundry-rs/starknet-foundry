use forge::running::CACHE_DIR_NAME;

use crate::e2e::common::runner::{
    runner, setup_package, setup_package_with_file_patterns, BASE_FILE_PATTERNS,
};
use indoc::indoc;

#[test]
fn without_cache() {
    let temp = setup_package("forking");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::test_fork_simple"])
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::test_fork_simple
        Running 0 test(s) from tests/
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
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

    snapbox
        .current_dir(&temp)
        .args(["--exact", "forking::test_fork_simple"])
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [FAIL] forking::test_fork_simple
        
        Failure data:
            original value: [1480335954842313548834020101284630397133856818], converted to a string: [Balance should be 2]
        
        Running 0 test(s) from tests/
        Tests: 0 passed, 1 failed, 0 skipped

        Failures:
            forking::test_fork_simple
        "#});
}

#[test]
fn with_clean_cache() {
    let temp = setup_package_with_file_patterns(
        "forking",
        &[BASE_FILE_PATTERNS, &[".snfoundry_cache/*.json"]].concat(),
    );
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .args(["--clean-cache", "--exact", "forking::test_fork_simple"])
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::test_fork_simple
        Running 0 test(s) from tests/
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn forking_and_cache() {
    let temp = setup_package("forking");

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("tests::")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from forking package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [PASS] tests::test_forking::latest
        Block number: [..], block hash: [..]
        [PASS] tests::test_forking::pending
        Block parent hash: [..]
        [PASS] tests::test_forking::number
        [PASS] tests::test_forking::hash
        Tests: 4 passed, 0 failed, 0 skipped
        "#});

    let expected_cache_dir_path = temp.path().join(CACHE_DIR_NAME);
    let cache_file_number =
        expected_cache_dir_path.join("http___188_34_188_184_9545_rpc_v0_4_313997.json");
    let cache_file_hash =
        expected_cache_dir_path.join("http___188_34_188_184_9545_rpc_v0_4_131d49f5c47a6dd3ac48d5e644f56ab3e9d73dfedf4292cb7007d573d679e0a.json");

    assert!(!std::fs::read(cache_file_number).unwrap().is_empty());
    assert!(!std::fs::read(cache_file_hash).unwrap().is_empty());
}
