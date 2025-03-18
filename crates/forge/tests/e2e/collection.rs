use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn collection_with_lib() {
    let temp = setup_package("collection_with_lib");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 17 test(s) from collection_with_lib package
        Running 12 test(s) from src/
        [PASS] collection_with_lib::fab::tests::test_simple [..]
        [PASS] collection_with_lib::fob::tests::test_simple [..]
        [PASS] collection_with_lib::tests::test_fib_in_lib [..]
        [PASS] collection_with_lib::tests::test_simple [..]
        [PASS] collection_with_lib::fib::tests::test_fab_in_fib [..]
        [PASS] collection_with_lib::fib::tests::test_fib [..]
        [PASS] collection_with_lib::fab::fab_impl::tests::test_fab [..]
        [PASS] collection_with_lib::fib::tests::test_fob_in_fib [..]
        [PASS] collection_with_lib::tests::test_fob_in_lib [..]
        [PASS] collection_with_lib::fab::fab_impl::tests::test_super [..]
        [PASS] collection_with_lib::fob::fob_impl::tests::test_fob [..]
        [PASS] collection_with_lib::fab::fab_impl::tests::test_how_does_this_work [..]
        Running 5 test(s) from tests/
        [PASS] collection_with_lib_tests::fab::fab_mod::test_fab [..]
        [PASS] collection_with_lib_tests::fibfabfob::test_fob [..]
        [PASS] collection_with_lib_tests::fab::test_fab [..]
        [PASS] collection_with_lib_tests::fibfabfob::test_fib [..]
        [PASS] collection_with_lib_tests::fibfabfob::test_fab [..]
        Tests: 17 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}

#[test]
fn collection_without_lib() {
    let temp = setup_package("collection_without_lib");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 17 test(s) from collection_without_lib package
        Running 12 test(s) from src/
        [PASS] collection_without_lib::fab::tests::test_simple [..]
        [PASS] collection_without_lib::fab::fab_impl::tests::test_super [..]
        [PASS] collection_without_lib::tests::test_simple [..]
        [PASS] collection_without_lib::fab::fab_impl::tests::test_fab [..]
        [PASS] collection_without_lib::fob::tests::test_simple [..]
        [PASS] collection_without_lib::fib::tests::test_fob_in_fib [..]
        [PASS] collection_without_lib::tests::test_fib_in_lib [..]
        [PASS] collection_without_lib::fib::tests::test_fab_in_fib [..]
        [PASS] collection_without_lib::tests::test_fob_in_lib [..]
        [PASS] collection_without_lib::fab::fab_impl::tests::test_how_does_this_work [..]
        [PASS] collection_without_lib::fob::fob_impl::tests::test_fob [..]
        [PASS] collection_without_lib::fib::tests::test_fib [..]
        Running 5 test(s) from tests/
        [PASS] collection_without_lib_integrationtest::fibfabfob::test_fab [..]
        [PASS] collection_without_lib_integrationtest::fab::fab_mod::test_fab [..]
        [PASS] collection_without_lib_integrationtest::fab::test_fab [..]
        [PASS] collection_without_lib_integrationtest::fibfabfob::test_fob [..]
        [PASS] collection_without_lib_integrationtest::fibfabfob::test_fib [..]
        Tests: 17 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}
