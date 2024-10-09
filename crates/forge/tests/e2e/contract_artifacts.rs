use crate::e2e::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::DocumentMut;

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
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

    assert!(temp
        .join("target/dev/simple_package_integrationtest.test.starknet_artifacts.json")
        .exists());
    assert!(temp
        .join("target/dev/simple_package_integrationtest_HelloStarknet.test.contract_class.json")
        .exists());

    assert!(temp
        .join("target/dev/simple_package_unittest.test.starknet_artifacts.json")
        .exists());
    assert!(temp
        .join("target/dev/simple_package_unittest_HelloStarknet.test.contract_class.json")
        .exists());

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
    Tests: 9 passed, 2 failed, 0 skipped, 2 ignored, 0 filtered out

    Failures:
        simple_package_integrationtest::test_simple::test_failing
        simple_package_integrationtest::test_simple::test_another_failing
    "},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn no_optimization_flag() {
    let temp = setup_package("erc20_package");
    let output = test_runner(&temp)
        .arg("--no-optimization")
        .assert()
        .success();

    assert!(temp
        .join("target/dev/erc20_package_ERC20.contract_class.json")
        .exists());
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from erc20_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] erc20_package_integrationtest::test_complex::complex[..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}
