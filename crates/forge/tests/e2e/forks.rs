use forge::running::CACHE_DIR_NAME;
use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_package};

#[test]
fn forks() {
    let temp = setup_package("forks");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from forks package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [PASS] tests::test_forks::latest
        Block number: [..], block hash: [..]
        [PASS] tests::test_forks::pending
        Block parent hash: [..]
        [PASS] tests::test_forks::number
        [PASS] tests::test_forks::hash
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
