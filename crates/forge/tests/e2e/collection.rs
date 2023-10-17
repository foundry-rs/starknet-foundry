use crate::assert_stdout_contains;
use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::e2e::common::runner::runner;

#[test]
fn collection_with_lib() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from(
        "tests/data/collection_with_lib",
        &["**/*.cairo", "**/*.toml"],
    )
    .unwrap();

    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 17 test(s) from collection_with_lib package
        Running 12 test(s) from src/
        [PASS] collection_with_lib::test_simple
        [PASS] collection_with_lib::test_fob_in_lib
        [PASS] collection_with_lib::test_fib_in_lib
        [PASS] collection_with_lib::fib::test_fib
        [PASS] collection_with_lib::fib::test_fob_in_fib
        [PASS] collection_with_lib::fib::test_fab_in_fib
        [PASS] collection_with_lib::fob::test_simple
        [PASS] collection_with_lib::fob::fob_impl::test_fob
        [PASS] collection_with_lib::fab::test_simple
        [PASS] collection_with_lib::fab::fab_impl::test_fab
        [PASS] collection_with_lib::fab::fab_impl::test_how_does_this_work
        [PASS] collection_with_lib::fab::fab_impl::test_super
        Running 5 test(s) from tests/
        [PASS] tests::fibfabfob::test_fib
        [PASS] tests::fibfabfob::test_fob
        [PASS] tests::fibfabfob::test_fab
        [PASS] tests::fab::test_fab
        [PASS] tests::fab::fab_mod::test_fab
        Tests: 17 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn collection_without_lib() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from(
        "tests/data/collection_without_lib",
        &["**/*.cairo", "**/*.toml"],
    )
    .unwrap();

    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().success();
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 17 test(s) from collection_without_lib package
        Running 12 test(s) from src/
        [PASS] collection_without_lib::test_simple
        [PASS] collection_without_lib::test_fob_in_lib
        [PASS] collection_without_lib::test_fib_in_lib
        [PASS] collection_without_lib::fib::test_fib
        [PASS] collection_without_lib::fib::test_fob_in_fib
        [PASS] collection_without_lib::fib::test_fab_in_fib
        [PASS] collection_without_lib::fob::test_simple
        [PASS] collection_without_lib::fob::fob_impl::test_fob
        [PASS] collection_without_lib::fab::test_simple
        [PASS] collection_without_lib::fab::fab_impl::test_fab
        [PASS] collection_without_lib::fab::fab_impl::test_how_does_this_work
        [PASS] collection_without_lib::fab::fab_impl::test_super
        Running 5 test(s) from tests/
        [PASS] tests::fab::test_fab
        [PASS] tests::fab::fab_mod::test_fab
        [PASS] tests::fibfabfob::test_fib
        [PASS] tests::fibfabfob::test_fob
        [PASS] tests::fibfabfob::test_fab
        Tests: 17 passed, 0 failed, 0 skipped
        "#}
    );
}
