use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::e2e::common::runner::runner;

#[test]
fn complex_structure() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from(
        "tests/data/complex_structure_test",
        &["**/*.cairo", "**/*.toml"],
    )
    .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 16 test(s) and 4 test file(s)
        Running 12 test(s) from src/lib.cairo
        [PASS] src::test_simple
        [PASS] src::test_fob_in_lib
        [PASS] src::test_fib_in_lib
        [PASS] src::fib::test_fib
        [PASS] src::fib::test_fob_in_fib
        [PASS] src::fib::test_fab_in_fib
        [PASS] src::fob::test_simple
        [PASS] src::fob::fob_impl::test_fob
        [PASS] src::fab::test_simple
        [PASS] src::fab::fab_impl::test_fab
        [PASS] src::fab::fab_impl::test_how_does_this_work
        [PASS] src::fab::fab_impl::test_super
        Running 1 test(s) from fob.cairo
        [PASS] fob::fob::test_fob
        Running 0 test(s) from tests/fab.cairo
        Running 3 test(s) from tests/fibfabfob.cairo
        [PASS] fibfabfob::fibfabfob::test_fib
        [PASS] fibfabfob::fibfabfob::test_fob
        [PASS] fibfabfob::fibfabfob::test_fab
        Tests: 16 passed, 0 failed, 0 skipped
        "#});
}
