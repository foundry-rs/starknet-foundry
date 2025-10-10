use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
#[cfg_attr(
    feature = "skip_test_for_only_latest_scarb",
    ignore = "Plugin checks skipped"
)]
fn simple_addition() {
    let temp = setup_package("test_case");

    let output = test_runner(&temp).arg("simple_addition").assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from test_case package
        Running 3 test(s) from tests/
        [PASS] test_case_integrationtest::single_attribute::simple_addition_1_2_3 [..]
        [PASS] test_case_integrationtest::single_attribute::simple_addition_3_4_7 [..]
        [PASS] test_case_integrationtest::single_attribute::simple_addition_5_6_11 [..]
        Running 0 test(s) from src/
        Tests: 3 passed, 0 failed, 0 ignored, [..] filtered out
        "},
    );
}

#[test]
#[cfg_attr(
    feature = "skip_test_for_only_latest_scarb",
    ignore = "Plugin checks skipped"
)]
fn with_exit_first_flag() {
    let temp = setup_package("test_case");

    let output = test_runner(&temp)
        .arg("test_fib_with_threshold")
        .arg("--exit-first")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from test_case package
        Running 2 test(s) from tests/
        [FAIL] test_case_integrationtest::exit_first::test_fib_with_threshold_0_1_3

        Failure data:
            "result should be greater than threshold"

        Tests: 0 passed, 1 failed, 0 ignored, [..] filtered out
        Interrupted execution of 1 test(s).

        Failures:
            test_case_integrationtest::exit_first::test_fib_with_threshold_0_1_3
        "#},
    );
}

#[test]
#[cfg_attr(
    feature = "skip_test_for_only_latest_scarb",
    ignore = "Plugin checks skipped"
)]
fn with_multiple_attributes() {
    let temp = setup_package("test_case");

    let output = test_runner(&temp)
        .arg("multiple_attributes")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [IGNORE] test_case_integrationtest::multiple_attributes::with_ignore_3_4_7
        [IGNORE] test_case_integrationtest::multiple_attributes::with_ignore_1_2_3
        [PASS] test_case_integrationtest::multiple_attributes::with_available_gas_3_4_7 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer_3_4 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer_different_order_3_4 [..]
        [FAIL] test_case_integrationtest::multiple_attributes::with_available_gas_exceed_limit_3_4_7

        Failure data:
        	Test cost exceeded the available gas. Consumed [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_available_gas_1_2_3 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_should_panic_3_4_7 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_should_panic_1_2_3 [..]
        [FAIL] test_case_integrationtest::multiple_attributes::with_available_gas_exceed_limit_1_2_3

        Failure data:
        	Test cost exceeded the available gas. Consumed [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer_1_2 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer_different_order_1_2 [..]
        [PASS] test_case_integrationtest::multiple_attributes::with_fuzzer_different_order [..]
        Running 0 test(s) from src/
        Tests: 10 passed, 2 failed, 2 ignored, [..] filtered out
        Fuzzer seed: [..]

        Failures:
            test_case_integrationtest::multiple_attributes::with_available_gas_exceed_limit_3_4_7
            test_case_integrationtest::multiple_attributes::with_available_gas_exceed_limit_1_2_3
        "},
    );
}

#[test]
#[cfg_attr(
    feature = "skip_test_for_only_latest_scarb",
    ignore = "Plugin checks skipped"
)]
fn addition_with_name_arg() {
    let temp = setup_package("test_case");

    let output = test_runner(&temp)
        .arg("addition_with_name_arg")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from test_case package
        Running 3 test(s) from tests/
        [PASS] test_case_integrationtest::single_attribute::addition_with_name_arg_one_and_two [..]
        [PASS] test_case_integrationtest::single_attribute::addition_with_name_arg_three_and_four [..]
        [PASS] test_case_integrationtest::single_attribute::addition_with_name_arg_five_and_six [..]
        Running 0 test(s) from src/
        Tests: 3 passed, 0 failed, 0 ignored, [..] filtered out
        "},
    );
}
