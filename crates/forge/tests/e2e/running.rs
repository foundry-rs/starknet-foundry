use super::common::runner::{get_current_branch, get_remote_url, setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{AsOutput, assert_stdout, assert_stdout_contains};
use std::{fs, str::FromStr};
use test_utils::tempdir_with_tool_versions;
use toml_edit::{DocumentMut, value};

#[test]
fn simple_package() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp).assert().code(1);

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
fn simple_package_with_git_dependency() {
    let temp = tempdir_with_tool_versions().unwrap();

    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();
    let remote_url = get_remote_url().to_lowercase();
    let branch = get_current_branch();
    let manifest_path = temp.child("Scarb.toml");
    manifest_path
        .write_str(&formatdoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[target.starknet-contract]]

            [dependencies]
            starknet = "2.6.4"
            snforge_std = {{ git = "https://github.com/{}", branch = "{}" }}
            "#,
            remote_url,
            branch
        ))
        .unwrap();

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        [..]Updating git repository https://github.com/{}
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
        ",
            remote_url.trim_end_matches(".git")
        ),
    );
}

#[test]
fn with_failing_scarb_build() {
    let temp = setup_package("simple_package");
    temp.child("src/lib.cairo")
        .write_str(indoc!(
            r"
                mod hello_starknet;
                mods erc20;
            "
        ))
        .unwrap();

    let output = test_runner(&temp).arg("--no-optimization").assert().code(2);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [ERROR] Failed to build contracts with Scarb: `scarb` exited with error
            "
        ),
    );
}

#[test]
fn with_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp).arg("two").assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from simple_package package
        Running 2 test(s) from tests/
        [PASS] simple_package_integrationtest::test_simple::test_two [..]
        [PASS] simple_package_integrationtest::test_simple::test_two_and_two [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 11 filtered out
        "},
    );
}

#[test]
fn with_filter_matching_module() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("ext_function_test::")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from simple_package package
        Running 3 test(s) from tests/
        [PASS] simple_package_integrationtest::ext_function_test::test_my_test [..]
        [IGNORE] simple_package_integrationtest::ext_function_test::ignored_test
        [PASS] simple_package_integrationtest::ext_function_test::test_simple [..]
        Tests: 2 passed, 0 failed, 0 skipped, 1 ignored, 10 filtered out
        "},
    );
}

#[test]
fn with_exact_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("simple_package_integrationtest::test_simple::test_two")
        .arg("--exact")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] simple_package_integrationtest::test_simple::test_two [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, other filtered out
        "},
    );
}

#[test]
fn with_skip_filter_matching_module() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("--skip")
        .arg("simple_package")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 0 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 0 test(s) from tests/
        Tests: 0 passed, 0 failed, 0 skipped, 0 ignored, 13 filtered out
        "},
    );
}

#[test]
fn with_skip_filter_matching_test_name() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("--skip")
        .arg("failing")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 11 test(s) from simple_package package
        Running 9 test(s) from tests/
        [IGNORE] simple_package_integrationtest::ext_function_test::ignored_test
        [PASS] simple_package_integrationtest::test_simple::test_two
        [PASS] simple_package_integrationtest::test_simple::test_two_and_two
        [PASS] simple_package_integrationtest::test_simple::test_simple2
        [PASS] simple_package_integrationtest::ext_function_test::test_simple
        [PASS] simple_package_integrationtest::without_prefix::five
        [PASS] simple_package_integrationtest::ext_function_test::test_my_test
        [PASS] simple_package_integrationtest::test_simple::test_simple
        [PASS] simple_package_integrationtest::contract::call_and_invoke
        Running 2 test(s) from src/
        [IGNORE] simple_package::tests::ignored_test
        [PASS] simple_package::tests::test_fib
        Tests: 9 passed, 0 failed, 0 skipped, 2 ignored, 0 filtered out
        "},
    );
}

#[test]
fn with_skip_filter_matching_multiple_test_name() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("--skip")
        .arg("failing")
        .arg("--skip")
        .arg("two")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 9 test(s) from simple_package package
        Running 7 test(s) from tests/
        [IGNORE] simple_package_integrationtest::ext_function_test::ignored_test
        [PASS] simple_package_integrationtest::test_simple::test_simple2
        [PASS] simple_package_integrationtest::ext_function_test::test_simple
        [PASS] simple_package_integrationtest::without_prefix::five
        [PASS] simple_package_integrationtest::ext_function_test::test_my_test
        [PASS] simple_package_integrationtest::test_simple::test_simple
        [PASS] simple_package_integrationtest::contract::call_and_invoke
        Running 2 test(s) from src/
        [IGNORE] simple_package::tests::ignored_test
        [PASS] simple_package::tests::test_fib
        Tests: 7 passed, 0 failed, 0 skipped, 2 ignored, 0 filtered out
        "},
    );
}

#[test]
fn with_exact_filter_and_duplicated_test_names() {
    let temp = setup_package("duplicated_test_names");

    let output = test_runner(&temp)
        .arg("duplicated_test_names_integrationtest::tests_a::test_simple")
        .arg("--exact")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from duplicated_test_names package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] duplicated_test_names_integrationtest::tests_a::test_simple [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, other filtered out
        "},
    );
}

#[test]
fn with_non_matching_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp).arg("qwerty").assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 0 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 0 test(s) from tests/
        Tests: 0 passed, 0 failed, 0 skipped, 0 ignored, 13 filtered out
        "},
    );
}

#[test]
fn with_ignored_flag() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp).arg("--ignored").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from simple_package package
        Running 1 test(s) from src/
        [PASS] simple_package::tests::ignored_test [..]
        Running 1 test(s) from tests/
        [FAIL] simple_package_integrationtest::ext_function_test::ignored_test

        Failure data:
            0x6e6f742070617373696e67 ('not passing')

        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out

        Failures:
            simple_package_integrationtest::ext_function_test::ignored_test
        "},
    );
}

#[test]
fn with_include_ignored_flag() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp).arg("--include-ignored").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 13 test(s) from simple_package package
        Running 2 test(s) from src/
        [PASS] simple_package::tests::test_fib [..]
        [PASS] simple_package::tests::ignored_test [..]
        Running 11 test(s) from tests/
        [PASS] simple_package_integrationtest::contract::call_and_invoke [..]
        [PASS] simple_package_integrationtest::ext_function_test::test_my_test [..]
        [FAIL] simple_package_integrationtest::ext_function_test::ignored_test

        Failure data:
            0x6e6f742070617373696e67 ('not passing')

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
        Tests: 10 passed, 3 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            simple_package_integrationtest::ext_function_test::ignored_test
            simple_package_integrationtest::test_simple::test_failing
            simple_package_integrationtest::test_simple::test_another_failing
        "},
    );
}

#[test]
fn with_ignored_flag_and_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("--ignored")
        .arg("ext_function_test::ignored_test")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [FAIL] simple_package_integrationtest::ext_function_test::ignored_test

        Failure data:
            0x6e6f742070617373696e67 ('not passing')

        Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 12 filtered out

        Failures:
            simple_package_integrationtest::ext_function_test::ignored_test
        "},
    );
}

#[test]
fn with_include_ignored_flag_and_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("--include-ignored")
        .arg("ignored_test")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from simple_package package
        Running 1 test(s) from src/
        [PASS] simple_package::tests::ignored_test [..]
        Running 1 test(s) from tests/
        [FAIL] simple_package_integrationtest::ext_function_test::ignored_test

        Failure data:
            0x6e6f742070617373696e67 ('not passing')

        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out

        Failures:
            simple_package_integrationtest::ext_function_test::ignored_test
        "},
    );
}

#[test]
fn with_rerun_failed_flag_without_cache() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp).arg("--rerun-failed").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 13 test(s) from simple_package package
        Running 2 test(s) from src/
        [PASS] simple_package::tests::test_fib [..]
        Running 11 test(s) from tests/
        [PASS] simple_package_integrationtest::contract::call_and_invoke [..]
        [PASS] simple_package_integrationtest::ext_function_test::test_my_test [..]

        [PASS] simple_package_integrationtest::ext_function_test::test_simple [..]
        [PASS] simple_package_integrationtest::test_simple::test_simple [..]
        [PASS] simple_package_integrationtest::test_simple::test_simple2 [..]
        [PASS] simple_package_integrationtest::test_simple::test_two [..]
        [PASS] simple_package_integrationtest::test_simple::test_two_and_two [..]
        [FAIL] simple_package_integrationtest::test_simple::test_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        [FAIL] simple_package_integrationtest::test_simple::test_another_failing

        [PASS] simple_package_integrationtest::without_prefix::five [..]
        Failures:
            simple_package_integrationtest::test_simple::test_failing
            simple_package_integrationtest::test_simple::test_another_failing
        [IGNORE] simple_package::tests::ignored_test
        [IGNORE] simple_package_integrationtest::ext_function_test::ignored_test
        Tests: 9 passed, 2 failed, 0 skipped, 2 ignored, 0 filtered out
        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        "},
    );
}

#[test]
fn with_rerun_failed_flag_and_name_filter() {
    let temp = setup_package("simple_package");

    test_runner(&temp).assert().code(1);

    let output = test_runner(&temp)
        .arg("--rerun-failed")
        .arg("test_another_failing")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from simple_package package
        Running 1 test(s) from tests/
        [FAIL] simple_package_integrationtest::test_simple::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 12 filtered out

        Failures:
            simple_package_integrationtest::test_simple::test_another_failing

        "},
    );
}

#[test]
fn with_rerun_failed_flag() {
    let temp = setup_package("simple_package");

    test_runner(&temp).assert().code(1);

    let output = test_runner(&temp).arg("--rerun-failed").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [FAIL] simple_package_integrationtest::test_simple::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        [FAIL] simple_package_integrationtest::test_simple::test_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 11 filtered out

        Failures:
            simple_package_integrationtest::test_simple::test_another_failing
            simple_package_integrationtest::test_simple::test_failing

        "},
    );
}

#[test]
fn with_panic_data_decoding() {
    let temp = setup_package("panic_decoding");

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 8 test(s) from panic_decoding package
        Running 8 test(s) from tests/
        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_panic_decoding2

        Failure data:
            0x80

        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_assert

        Failure data:
            "assertion failed: `x`."

        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_panic_decoding

        Failure data:
            (0x7b ('{'), 0x616161 ('aaa'), 0x800000000000011000000000000000000000000000000000000000000000000, 0x98, 0x7c ('|'), 0x95)

        [PASS] panic_decoding_integrationtest::test_panic_decoding::test_simple2 (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [PASS] panic_decoding_integrationtest::test_panic_decoding::test_simple (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_assert_eq

        Failure data:
            "assertion `x == y` failed.
            x: 5
            y: 6"

        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_assert_message

        Failure data:
            "Another identifiable and meaningful error message"

        [FAIL] panic_decoding_integrationtest::test_panic_decoding::test_assert_eq_message

        Failure data:
            "assertion `x == y` failed: An identifiable and meaningful error message
            x: 5
            y: 6"

        Tests: 2 passed, 6 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            panic_decoding_integrationtest::test_panic_decoding::test_panic_decoding2
            panic_decoding_integrationtest::test_panic_decoding::test_assert
            panic_decoding_integrationtest::test_panic_decoding::test_panic_decoding
            panic_decoding_integrationtest::test_panic_decoding::test_assert_eq
            panic_decoding_integrationtest::test_panic_decoding::test_assert_message
            panic_decoding_integrationtest::test_panic_decoding::test_assert_eq_message
        "#},
    );
}

#[test]
fn with_exit_first() {
    let temp = setup_package("exit_first");
    let scarb_path = temp.child("Scarb.toml");

    scarb_path
        .write_str(&formatdoc!(
            r#"
            [package]
            name = "exit_first"
            version = "0.1.0"

            [dependencies]
            starknet = "2.4.0"
            snforge_std = {{ path = "{}" }}

            [tool.snforge]
            exit_first = true
            "#,
            Utf8PathBuf::from_str("../../snforge_std")
                .unwrap()
                .canonicalize_utf8()
                .unwrap()
                .to_string()
                .replace('\\', "/")
        ))
        .unwrap();

    let output = test_runner(&temp).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from exit_first package
        Running 2 test(s) from tests/
        [FAIL] exit_first_integrationtest::ext_function_test::simple_test

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 0 filtered out

        Failures:
            exit_first_integrationtest::ext_function_test::simple_test
        "},
    );
}

#[test]
fn with_exit_first_flag() {
    let temp = setup_package("exit_first");

    let output = test_runner(&temp).arg("--exit-first").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from exit_first package
        Running 2 test(s) from tests/
        [FAIL] exit_first_integrationtest::ext_function_test::simple_test

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 0 filtered out

        Failures:
            exit_first_integrationtest::ext_function_test::simple_test
        "},
    );
}

#[test]
fn should_panic() {
    let temp = setup_package("should_panic_test");

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! { r"
        Collected 14 test(s) from should_panic_test package
        Running 0 test(s) from src/
        Running 14 test(s) from tests/
        [FAIL] should_panic_test_integrationtest::should_panic_test::didnt_expect_panic

        Failure data:
            0x756e65787065637465642070616e6963 ('unexpected panic')

        [FAIL] should_panic_test_integrationtest::should_panic_test::should_panic_expected_contains_error

        Failure data:
            Incorrect panic data
            Actual:    [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x77696c6c, 0x4] (will)
            Expected:  [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x546869732077696c6c2070616e6963, 0xf] (This will panic)

        [FAIL] should_panic_test_integrationtest::should_panic_test::should_panic_byte_array_with_felt

        Failure data:
            Incorrect panic data
            Actual:    [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x546869732077696c6c2070616e6963, 0xf] (This will panic)
            Expected:  [0x546869732077696c6c2070616e6963] (This will panic)

        [FAIL] should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt_with_expected_multiple

        Failure data:
            Expected to panic, but no panic occurred
            Expected panic data:  [0x70616e6963206d657373616765, 0x7365636f6e64206d657373616765] (panic message, second message)

        [FAIL] should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt

        Failure data:
            Expected to panic, but no panic occurred

        [PASS] should_panic_test_integrationtest::should_panic_test::should_panic_no_data (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])

        [PASS] should_panic_test_integrationtest::should_panic_test::should_panic_check_data (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [FAIL] should_panic_test_integrationtest::should_panic_test::should_panic_not_matching_suffix

        Failure data:
            Incorrect panic data
            Actual:    [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x546869732077696c6c2070616e6963, 0xf] (This will panic)
            Expected:  [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x77696c6c2070616e696363, 0xb] (will panicc)

        [PASS] should_panic_test_integrationtest::should_panic_test::should_panic_match_suffix (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [PASS] should_panic_test_integrationtest::should_panic_test::should_panic_felt_matching (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [FAIL] should_panic_test_integrationtest::should_panic_test::should_panic_felt_with_byte_array

        Failure data:
            Incorrect panic data
            Actual:    [0x546869732077696c6c2070616e6963] (This will panic)
            Expected:  [0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0, 0x546869732077696c6c2070616e6963, 0xf] (This will panic)

        [PASS] should_panic_test_integrationtest::should_panic_test::should_panic_multiple_messages (l1_gas: [..], l1_data_gas: [..], l2_gas: [..])
        [FAIL] should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt_with_expected

        Failure data:
            Expected to panic, but no panic occurred
            Expected panic data:  [0x70616e6963206d657373616765] (panic message)

        [FAIL] should_panic_test_integrationtest::should_panic_test::should_panic_with_non_matching_data

        Failure data:
            Incorrect panic data
            Actual:    [0x6661696c696e6720636865636b] (failing check)
            Expected:  [0x0] ()

        Tests: 5 passed, 9 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            should_panic_test_integrationtest::should_panic_test::didnt_expect_panic
            should_panic_test_integrationtest::should_panic_test::should_panic_expected_contains_error
            should_panic_test_integrationtest::should_panic_test::should_panic_byte_array_with_felt
            should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt_with_expected_multiple
            should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt
            should_panic_test_integrationtest::should_panic_test::should_panic_not_matching_suffix
            should_panic_test_integrationtest::should_panic_test::should_panic_felt_with_byte_array
            should_panic_test_integrationtest::should_panic_test::expected_panic_but_didnt_with_expected
            should_panic_test_integrationtest::should_panic_test::should_panic_with_non_matching_data
        "},
    );
}

#[test]
#[ignore = "TODO(#3322) restore the asserted message to be proper test output and not `ERROR` after there exists a previous plugin version compatible with changes from #3027"]
fn incompatible_snforge_std_version_warning() {
    let temp = setup_package("steps");
    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["dev-dependencies"]["snforge_std"] = value("0.34.1");
    scarb_toml["dev-dependencies"]["snforge_scarb_plugin"] = value("0.34.1");
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        [WARNING] Package snforge_std version does not meet the recommended version requirement ^0.[..], [..]
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from steps package
        Running 2 test(s) from src/
        [PASS] steps::tests::steps_less_than_10000000 [..]
        [FAIL] steps::tests::steps_more_than_10000000

        Failure data:
            Could not reach the end of the program. RunResources has no remaining steps.
            Suggestion: Consider using the flag `--max-n-steps` to increase allowed limit of steps

        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            steps::tests::steps_more_than_10000000
        "},
    );
}

#[test]
fn incompatible_snforge_std_version_error() {
    let temp = setup_package("steps");
    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["dev-dependencies"]["snforge_std"] = value("0.42.0");
    scarb_toml["dev-dependencies"]["snforge_scarb_plugin"] = value("0.42.0");
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        [ERROR] Package snforge_std version does not meet the minimum required version >=0.44.0. Please upgrade snforge_std in Scarb.toml
        "},
    );
}

#[test]
fn detailed_resources_flag() {
    let temp = setup_package("erc20_package");
    let output = test_runner(&temp)
        .arg("--detailed-resources")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from erc20_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] erc20_package_integrationtest::test_complex::complex[..]
                steps: [..]
                memory holes: [..]
                builtins: ([..])
                syscalls: ([..])
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn detailed_resources_flag_sierra_gas() {
    let temp = setup_package("erc20_package");
    let output = test_runner(&temp)
        .arg("--detailed-resources")
        .arg("--tracked-resource")
        .arg("sierra-gas")
        .assert()
        .success();

    // Extra check to ensure that the output does not contain VM resources
    assert!(!output.as_stdout().contains("steps:"));

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 1 test(s) from erc20_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] erc20_package_integrationtest::test_complex::complex[..]
                sierra_gas_consumed: [..]
                syscalls: ([..])
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn detailed_resources_mixed_resources() {
    let temp = setup_package("forking");
    let output = test_runner(&temp)
        .arg("test_track_resources")
        .arg("--detailed-resources")
        .arg("--tracked-resource")
        .arg("sierra-gas")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]
        [WARNING] When tracking sierra gas and executing contracts with a Sierra version older than 1.7.0, syscall related resources may be incorrectly reported to the wrong resource type in the output of `--detailed-resources` flag.
        Collected 1 test(s) from forking package
        Running 1 test(s) from src/
        [PASS] forking::tests::test_track_resources [..]
                sierra_gas_consumed: [..]
                syscalls: ([..])
                steps: [..]
                memory holes: [..]
                builtins: (range_check: [..])

        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, [..] filtered out
        "},
    );
}

#[test]
fn catch_runtime_errors() {
    let temp = setup_package("simple_package");

    let expected_panic = "No such file or directory";

    temp.child("tests/test.cairo")
        .write_str(
            formatdoc!(
                r#"
                use snforge_std::fs::{{FileTrait, read_txt}};

                #[test]
                #[should_panic(expected: "{}")]
                fn catch_no_such_file() {{
                    let file = FileTrait::new("no_way_this_file_exists");
                    let content = read_txt(@file);

                    assert!(false);
                }}
            "#,
                expected_panic
            )
            .as_str(),
        )
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]
                [PASS] simple_package_integrationtest::test::catch_no_such_file [..]
            "
        ),
    );
}

#[test]
fn call_nonexistent_selector() {
    let temp = setup_package("nonexistent_selector");

    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        Collected 1 test(s) from nonexistent_selector package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] nonexistent_selector_integrationtest::test_contract::test_unwrapped_call_contract_syscall [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_9_1"), ignore)]
fn sierra_gas_with_older_scarb() {
    let temp = setup_package("erc20_package");
    let output = test_runner(&temp)
        .arg("--detailed-resources")
        .arg("--tracked-resource")
        .arg("sierra-gas")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        Checking requirements
        [..]Scarb Version [..] doesn't satisfy minimal 2.10.0[..]
        [..]To track sierra gas, minimal required scarb version is 2.10.0 (it comes with sierra >= 1.7.0 support)[..]
        [..]Follow instructions from https://docs.swmansion.com/scarb/download.html[..]
        [..]
        [ERROR] Requirements not satisfied
        "},
    );
}

#[test]
fn exact_printing_pass() {
    let temp = setup_package("deterministic_output");

    let output = test_runner(&temp).arg("pass").assert().code(0);

    assert_stdout(
        output,
        indoc! {r"
        Collected 2 test(s) from deterministic_output package
        Running 2 test(s) from src/
        [PASS] deterministic_output::test::first_test_pass_y [..]
        [PASS] deterministic_output::test::second_test_pass_x [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 2 filtered out
        "},
    );
}

#[test]
fn exact_printing_fail() {
    let temp = setup_package("deterministic_output");

    let output = test_runner(&temp).arg("fail").assert().code(1);

    assert_stdout(
        output,
        indoc! {r"
        Collected 2 test(s) from deterministic_output package
        Running 2 test(s) from src/
        [FAIL] deterministic_output::test::first_test_fail_x

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        [FAIL] deterministic_output::test::second_test_fail_y

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 2 filtered out

        Failures:
            deterministic_output::test::first_test_fail_x
            deterministic_output::test::second_test_fail_y
        "},
    );
}

#[test]
fn exact_printing_mixed() {
    let temp = setup_package("deterministic_output");

    let output = test_runner(&temp).arg("x").assert().code(1);

    assert_stdout(
        output,
        indoc! {r"
        Collected 2 test(s) from deterministic_output package
        Running 2 test(s) from src/
        [FAIL] deterministic_output::test::first_test_fail_x

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        [PASS] deterministic_output::test::second_test_pass_x [..]
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 2 filtered out

        Failures:
            deterministic_output::test::first_test_fail_x
        "},
    );
}

#[test]
fn dispatchers() {
    let temp = setup_package("dispatchers");

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        Collected 4 test(s) from dispatchers package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [FAIL] dispatchers_integrationtest::test::test_unrecoverable_not_possible_to_handle
        Failure data:
        Got an exception while executing a hint: Requested contract address [..] is not deployed.

        [PASS] dispatchers_integrationtest::test::test_error_handled_in_contract [..]
        [PASS] dispatchers_integrationtest::test::test_handle_and_panic [..]
        [PASS] dispatchers_integrationtest::test::test_handle_recoverable_in_test [..]
        Tests: 3 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            dispatchers_integrationtest::test::test_unrecoverable_not_possible_to_handle
        "},
    );
}
