use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::common::runner::runner;

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
        .stdout_matches(indoc! {r#"Collected 8 test(s) and 6 test file(s)
        Running 1 test(s) from src/fab/fab_impl.cairo
        [PASS] fab_impl::fab_impl::test_fab
        Running 1 test(s) from src/fab.cairo
        [PASS] fab::fab::test_simple
        Running 1 test(s) from src/fib.cairo
        [PASS] fib::fib::test_fib
        Running 1 test(s) from src/fob/fob_impl.cairo
        [PASS] fob_impl::fob_impl::test_fob
        Running 1 test(s) from src/fob.cairo
        [PASS] fob::fob::test_simple
        Running 3 test(s) from src/lib.cairo
        [PASS] src::test_simple
        [PASS] src::test_fob_in_lib
        [PASS] src::test_fib_in_lib
        Tests: 8 passed, 0 failed, 0 skipped
            "#});
}
