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
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 15 test(s) and 3 test file(s) from test_multiple package
        Running 12 inline test(s)
        [PASS] test_multiple::test_simple
        [PASS] test_multiple::test_fob_in_lib
        [PASS] test_multiple::test_fib_in_lib
        [PASS] test_multiple::fib::test_fib
        [PASS] test_multiple::fib::test_fob_in_fib
        [PASS] test_multiple::fib::test_fab_in_fib
        [PASS] test_multiple::fob::test_simple
        [PASS] test_multiple::fob::fob_impl::test_fob
        [PASS] test_multiple::fab::test_simple
        [PASS] test_multiple::fab::fab_impl::test_fab
        [PASS] test_multiple::fab::fab_impl::test_how_does_this_work
        [PASS] test_multiple::fab::fab_impl::test_super
        Running 0 test(s) from tests/fab.cairo
        Running 3 test(s) from tests/fibfabfob.cairo
        [PASS] fibfabfob::test_fib
        [PASS] fibfabfob::test_fob
        [PASS] fibfabfob::test_fab
        Tests: 15 passed, 0 failed, 0 skipped
        "#});
}
