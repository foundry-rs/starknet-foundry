use crate::e2e::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::DocumentMut;

#[test]
fn unit_and_integration() {
    let temp = setup_package("targets/unit_and_integration");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from unit_and_integration package
    Running 1 test(s) from tests/
    [PASS] unit_and_integration_integrationtest::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 1 test(s) from src/
    [PASS] unit_and_integration::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn unit_and_lib_integration() {
    let temp = setup_package("targets/unit_and_lib_integration");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from unit_and_lib_integration package
    Running 1 test(s) from tests/
    [PASS] unit_and_lib_integration_tests::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 1 test(s) from src/
    [PASS] unit_and_lib_integration::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn only_integration() {
    let temp = setup_package("targets/only_integration");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from only_integration package
    Running 1 test(s) from tests/
    [PASS] only_integration_integrationtest::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 0 test(s) from src/
    Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn only_unit() {
    let temp = setup_package("targets/only_unit");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from only_unit package
    Running 1 test(s) from src/
    [PASS] only_unit::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn only_lib_integration() {
    let temp = setup_package("targets/only_lib_integration");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from only_lib_integration package
    Running 1 test(s) from tests/
    [PASS] only_lib_integration_tests::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 0 test(s) from src/
    Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn with_features() {
    let temp = setup_package("targets/with_features");
    let output = test_runner(&temp)
        .arg("--features")
        .arg("enable_for_tests")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from with_features package
    Running 1 test(s) from tests/
    [PASS] with_features_integrationtest::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 1 test(s) from src/
    [PASS] with_features::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
fn with_features_fails_without_flag() {
    let temp = setup_package("targets/with_features");
    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from with_features package
    Running 1 test(s) from tests/
    [FAIL] with_features_integrationtest::tests::declare_and_call_contract_from_lib

    Failure data:
        "Failed to get contract artifact for name = HelloStarknet."

    Running 1 test(s) from src/
    [FAIL] with_features::tests::declare_contract_from_lib

    Failure data:
        "Failed to get contract artifact for name = HelloStarknet."

    Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out

    Failures:
        with_features_integrationtest::tests::declare_and_call_contract_from_lib
        with_features::tests::declare_contract_from_lib
    "#},
    );
}

#[test]
// Case: We define custom test target for both unit and integration test types
// We do not define `build-external-contracts = ["targets::*"]` for `integration` target
// The test still passes because contracts are collected from `unit` target which includes
// the contracts from package by the default
fn custom_target() {
    let temp = setup_package("targets/custom_target");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from custom_target package
    Running 1 test(s) from tests/
    [PASS] custom_target_integrationtest::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 1 test(s) from src/
    [PASS] custom_target::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
// Case: We define custom test target for both unit and integration test types
// We do not define `build-external-contracts = ["targets::*"]` for `integration` target
// The test still passes because contracts are collected from `unit` target which includes
// the contracts from package by the default
fn custom_target_custom_names() {
    let temp = setup_package("targets/custom_target_custom_names");
    let output = test_runner(&temp).assert().code(0);

    // Scarb will use the name of the package for unit tests even if custom
    // name for the unit test target is defined
    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 2 test(s) from custom_target_custom_names package
    Running 1 test(s) from tests/
    [PASS] custom_first::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Running 1 test(s) from src/
    [PASS] custom_target_custom_names::tests::declare_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
// Case: We define custom test target for both unit and integration test types
// We must `build-external-contracts = ["targets::*"]` for `integration` target otherwise
// they will not be built and included for declaring.
fn custom_target_only_integration() {
    let temp = setup_package("targets/custom_target_only_integration");
    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from custom_target_only_integration package
    Running 1 test(s) from tests/
    [PASS] custom_first::tests::declare_and_call_contract_from_lib (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
    Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "},
    );
}

#[test]
// Case: We define custom test target for integration test type
// We delete `build-external-contracts = ["targets::*"]` for `integration` so the test fails
fn custom_target_only_integration_without_external() {
    let temp = setup_package("targets/custom_target_only_integration");

    // Remove `build-external-contracts` from `[[test]]` target
    let manifest_path = temp.child("Scarb.toml");
    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    let test_target = scarb_toml["test"].as_array_of_tables_mut().unwrap();
    assert_eq!(test_target.len(), 1);
    let test_target = test_target.get_mut(0).unwrap();
    test_target.remove("build-external-contracts").unwrap();
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from custom_target_only_integration package
    Running 1 test(s) from tests/
    [FAIL] custom_first::tests::declare_and_call_contract_from_lib

    Failure data:
        "Failed to get contract artifact for name = HelloStarknet."

    Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
    "#},
    );
}

#[test]
fn simple_package_no_starknet_contract_target() {
    let temp = setup_package("simple_package");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml.as_table_mut().remove("target");
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().code(1);

    assert!(
        temp.join("target/dev/simple_package_integrationtest.test.starknet_artifacts.json")
            .exists()
    );
    assert!(
        temp.join(
            "target/dev/simple_package_integrationtest_HelloStarknet.test.contract_class.json"
        )
        .exists()
    );

    assert!(
        temp.join("target/dev/simple_package_unittest.test.starknet_artifacts.json")
            .exists()
    );
    assert!(
        temp.join("target/dev/simple_package_unittest_HelloStarknet.test.contract_class.json")
            .exists()
    );

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 13 test(s) from simple_package package
    Running 2 test(s) from src/
    [PASS] simple_package::tests::test_fib [..]
    [IGNORE] simple_package::tests::ignored_test
    Running 11 test(s) from tests/
    [PASS] simple_package_integrationtest::contract::call_and_invoke [..]
    [PASS] simple_package_integrationtest::ext_function_test::test_my_test [..]
    [IGNORE] simple_package_integrationtest::ext_function_test::ignored_test
    [PASS] simple_package_integrationtest::ext_function_test::test_simple [..]
    [PASS] simple_package_integrationtest::test_simple::test_simple [..]
    [PASS] simple_package_integrationtest::test_simple::test_simple2 [..]
    [PASS] simple_package_integrationtest::test_simple::test_two [..]
    [PASS] simple_package_integrationtest::test_simple::test_two_and_two [..]
    [FAIL] simple_package_integrationtest::test_simple::test_failing

    Failure data:
        0x6661696c696e6720636865636b ('failing check')

    [FAIL] simple_package_integrationtest::test_simple::test_another_failing

    Failure data:
        0x6661696c696e6720636865636b ('failing check')

    [PASS] simple_package_integrationtest::without_prefix::five [..]
    Tests: 9 passed, 2 failed, 0 skipped, 2 ignored, 0 excluded, 0 filtered out

    Failures:
        simple_package_integrationtest::test_simple::test_failing
        simple_package_integrationtest::test_simple::test_another_failing
    "},
    );
}

#[test]
fn no_optimization_flag() {
    let temp = setup_package("erc20_package");
    let output = test_runner(&temp)
        .arg("--no-optimization")
        .assert()
        .success();

    assert!(
        temp.join("target/dev/erc20_package_ERC20.contract_class.json")
            .exists()
    );
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from erc20_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] erc20_package_integrationtest::test_complex::complex[..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}
