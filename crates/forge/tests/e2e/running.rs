use super::common::runner::{
    get_current_branch, get_remote_url, runner, setup_package, test_runner,
};
use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use camino::Utf8PathBuf;
use forge::CAIRO_EDITION;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::assert_stdout_contains;
use std::{fs, path::Path, str::FromStr};
use test_utils::tempdir_with_tool_versions;
use toml_edit::{value, DocumentMut, Item};

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
    [PASS] tests::contract::call_and_invoke [..]
    [PASS] tests::ext_function_test::test_my_test [..]
    [IGNORE] tests::ext_function_test::ignored_test
    [PASS] tests::ext_function_test::test_simple [..]
    [PASS] tests::test_simple::test_simple [..]
    [PASS] tests::test_simple::test_simple2 [..]
    [PASS] tests::test_simple::test_two [..]
    [PASS] tests::test_simple::test_two_and_two [..]
    [FAIL] tests::test_simple::test_failing
    
    Failure data:
        0x6661696c696e6720636865636b ('failing check')
    
    [FAIL] tests::test_simple::test_another_failing
    
    Failure data:
        0x6661696c696e6720636865636b ('failing check')
    
    [PASS] tests::without_prefix::five [..]
    Tests: 9 passed, 2 failed, 0 skipped, 2 ignored, 0 filtered out
    
    Failures:
        tests::test_simple::test_failing
        tests::test_simple::test_another_failing
    "},
    );
}

#[test]
fn simple_package_with_git_dependency() {
    let temp = tempdir_with_tool_versions().unwrap();

    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();
    let remote_url = get_remote_url();
    let branch = get_current_branch();
    let manifest_path = temp.child("Scarb.toml");
    manifest_path
        .write_str(&formatdoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[target.starknet-contract]]
            sierra = true
            casm = true

            [dependencies]
            starknet = "2.5.4"
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
        [PASS] tests::contract::call_and_invoke [..]
        [PASS] tests::ext_function_test::test_my_test [..]
        [IGNORE] tests::ext_function_test::ignored_test
        [PASS] tests::ext_function_test::test_simple [..]
        [PASS] tests::test_simple::test_simple [..]
        [PASS] tests::test_simple::test_simple2 [..]
        [PASS] tests::test_simple::test_two [..]
        [PASS] tests::test_simple::test_two_and_two [..]
        [FAIL] tests::test_simple::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] tests::test_simple::test_another_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [PASS] tests::without_prefix::five [..]
        Tests: 9 passed, 2 failed, 0 skipped, 2 ignored, 0 filtered out
        
        Failures:
            tests::test_simple::test_failing
            tests::test_simple::test_another_failing
        ",
            remote_url.trim_end_matches(".git")
        ),
    );
}

#[test]
fn with_failing_scarb_build() {
    let temp = setup_package("simple_package");
    let lib_file = temp.child("src/lib.cairo");
    lib_file
        .write_str(indoc!(
            r"
        mod hello_starknet;
        mods erc20;
    "
        ))
        .unwrap();

    test_runner(&temp).assert().code(2).stdout_eq(indoc! {r"
            [ERROR] Failed to build test artifacts with Scarb: `scarb` exited with error
        "});
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
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::test_simple::test_two [..]
        [PASS] tests::test_simple::test_two_and_two [..]
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
        Running 0 test(s) from src/
        Running 3 test(s) from tests/
        [PASS] tests::ext_function_test::test_my_test [..]
        [IGNORE] tests::ext_function_test::ignored_test
        [PASS] tests::ext_function_test::test_simple [..]
        Tests: 2 passed, 0 failed, 0 skipped, 1 ignored, 10 filtered out
        "},
    );
}

#[test]
fn with_exact_filter() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("tests::test_simple::test_two")
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
        [PASS] tests::test_simple::test_two [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 12 filtered out
        "},
    );
}
#[test]
fn with_gas_usage() {
    let temp = setup_package("simple_package");

    let output = test_runner(&temp)
        .arg("tests::test_simple::test_two")
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
        [PASS] tests::test_simple::test_two (gas: ~1)
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 12 filtered out
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
        [FAIL] tests::ext_function_test::ignored_test
        
        Failure data:
            0x6e6f742070617373696e67 ('not passing')
        
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out
        
        Failures:
            tests::ext_function_test::ignored_test
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
        [PASS] tests::contract::call_and_invoke [..]
        [PASS] tests::ext_function_test::test_my_test [..]
        [FAIL] tests::ext_function_test::ignored_test
        
        Failure data:
            0x6e6f742070617373696e67 ('not passing')
        
        [PASS] tests::ext_function_test::test_simple [..]
        [PASS] tests::test_simple::test_simple [..]
        [PASS] tests::test_simple::test_simple2 [..]
        [PASS] tests::test_simple::test_two [..]
        [PASS] tests::test_simple::test_two_and_two [..]
        [FAIL] tests::test_simple::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] tests::test_simple::test_another_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [PASS] tests::without_prefix::five [..]
        Tests: 10 passed, 3 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            tests::ext_function_test::ignored_test
            tests::test_simple::test_failing
            tests::test_simple::test_another_failing
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
        [FAIL] tests::ext_function_test::ignored_test
 
        Failure data:
            0x6e6f742070617373696e67 ('not passing')
        
        Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 12 filtered out
        
        Failures:
            tests::ext_function_test::ignored_test
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
        [FAIL] tests::ext_function_test::ignored_test
        
        Failure data:
            0x6e6f742070617373696e67 ('not passing')

        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out
        
        Failures:
            tests::ext_function_test::ignored_test
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
        [PASS] tests::contract::call_and_invoke [..]
        [PASS] tests::ext_function_test::test_my_test [..]

        [PASS] tests::ext_function_test::test_simple [..]
        [PASS] tests::test_simple::test_simple [..]
        [PASS] tests::test_simple::test_simple2 [..]
        [PASS] tests::test_simple::test_two [..]
        [PASS] tests::test_simple::test_two_and_two [..]
        [FAIL] tests::test_simple::test_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        [FAIL] tests::test_simple::test_another_failing

        [PASS] tests::without_prefix::five [..]
        Failures:
            tests::test_simple::test_failing
            tests::test_simple::test_another_failing
        [IGNORE] simple_package::tests::ignored_test
        [IGNORE] tests::ext_function_test::ignored_test
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
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [FAIL] tests::test_simple::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 12 filtered out

        Failures:
            tests::test_simple::test_another_failing

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
        [FAIL] tests::test_simple::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        [FAIL] tests::test_simple::test_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')

        Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 11 filtered out

        Failures:
            tests::test_simple::test_another_failing
            tests::test_simple::test_failing

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
        Running 0 test(s) from src/
        Running 8 test(s) from tests/
        [FAIL] tests::test_panic_decoding::test_panic_decoding2
        
        Failure data:
            0x80
        
        [FAIL] tests::test_panic_decoding::test_assert
        
        Failure data:
            "assertion failed: `x`."
        
        [FAIL] tests::test_panic_decoding::test_panic_decoding
        
        Failure data:
            (0x7b ('{'), 0x616161 ('aaa'), 0x800000000000011000000000000000000000000000000000000000000000000, 0x98, 0x7c ('|'), 0x95)
        
        [PASS] tests::test_panic_decoding::test_simple2 (gas: ~1)
        [PASS] tests::test_panic_decoding::test_simple (gas: ~1)
        [FAIL] tests::test_panic_decoding::test_assert_eq
        
        Failure data:
            "assertion `x == y` failed.
            x: 5
            y: 6"
        
        [FAIL] tests::test_panic_decoding::test_assert_message
        
        Failure data:
            "Another identifiable and meaningful error message"
        
        [FAIL] tests::test_panic_decoding::test_assert_eq_message
        
        Failure data:
            "assertion `x == y` failed: An identifiable and meaningful error message
            x: 5
            y: 6"
        
        Tests: 2 passed, 6 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            tests::test_panic_decoding::test_panic_decoding2
            tests::test_panic_decoding::test_assert
            tests::test_panic_decoding::test_panic_decoding
            tests::test_panic_decoding::test_assert_eq
            tests::test_panic_decoding::test_assert_message
            tests::test_panic_decoding::test_assert_eq_message
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

            [[target.starknet-contract]]
            sierra = true
            casm = true

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
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [FAIL] tests::ext_function_test::simple_test

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 0 filtered out

        Failures:
            tests::ext_function_test::simple_test
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
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [FAIL] tests::ext_function_test::simple_test

        Failure data:
            0x73696d706c6520636865636b ('simple check')

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 0 filtered out

        Failures:
            tests::ext_function_test::simple_test
        "},
    );
}

#[test]
fn init_new_project_test() {
    let temp = tempdir_with_tool_versions().unwrap();

    runner(&temp).args(["init", "test_name"]).assert().success();
    let manifest_path = temp.child("test_name/Scarb.toml");

    let generated_toml = std::fs::read_to_string(manifest_path.path()).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let expected_toml = formatdoc!(
        r#"
            [package]
            name = "test_name"
            version = "0.1.0"
            edition = "{}"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            starknet = "2.5.4"

            [dev-dependencies]
            snforge_std = {{ git = "https://github.com/foundry-rs/starknet-foundry", tag = "v{}" }}

            [[target.starknet-contract]]
            casm = true
        "#,
        CAIRO_EDITION,
        version,
    );

    assert_eq!(generated_toml, expected_toml);

    let remote_url = get_remote_url();
    let branch = get_current_branch();
    manifest_path
        .write_str(&formatdoc!(
            r#"
        [package]
        name = "test_name"
        version = "0.1.0"

        [[target.starknet-contract]]
        casm = true

        [dependencies]
        starknet = "2.5.4"

        [dev-dependencies]
        snforge_std = {{ git = "https://github.com/{}", branch = "{}" }}
        "#,
            remote_url,
            branch
        ))
        .unwrap();

    // Check if template works with current version of snforge_std
    let output = test_runner(&temp)
        .current_dir(temp.child(Path::new("test_name")))
        .assert()
        .success();
    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        [..]Updating git repository https://github.com/{}
        [..]Compiling test_name v0.1.0[..]
        [..]Finished[..]


        Collected 2 test(s) from test_name package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::test_contract::test_increase_balance [..]
        [PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
    ",
            remote_url.trim_end_matches(".git")
        ),
    );
}

#[test]
fn should_panic() {
    let temp = tempdir_with_tool_versions().unwrap();
    temp.copy_from("tests/data/should_panic_test", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc! { r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 8 test(s) from should_panic_test package
        Running 0 test(s) from src/
        Running 8 test(s) from tests/
        [FAIL] tests::should_panic_test::expected_panic_but_didnt
        
        Failure data:
            Expected to panic but didn't

        [PASS] tests::should_panic_test::should_panic_check_data [..]
        [PASS] tests::should_panic_test::should_panic_multiple_messages [..]
        [PASS] tests::should_panic_test::should_panic_no_data [..]
        
        Success data:
            0x0 ('')
        
        [FAIL] tests::should_panic_test::should_panic_with_non_matching_data
        
        Failure data:
            Incorrect panic data
            Actual:    [8111420071579136082810415440747] (failing check)
            Expected:  [0] ()
        
        [FAIL] tests::should_panic_test::expected_panic_but_didnt_with_expected
        
        Failure data:
            Expected to panic but didn't
            Expected panic data:  [8903707727067478891290643490661] (panic message)
        
        [FAIL] tests::should_panic_test::expected_panic_but_didnt_with_expected_multiple
        
        Failure data:
            Expected to panic but didn't
            Expected panic data:  [8903707727067478891290643490661, 2340509922561928411394884117817189] (panic message, second message)
        
        [FAIL] tests::should_panic_test::didnt_expect_panic
        
        Failure data:
            0x756e65787065637465642070616e6963 ('unexpected panic')
        
        Tests: 3 passed, 5 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            tests::should_panic_test::expected_panic_but_didnt
            tests::should_panic_test::should_panic_with_non_matching_data
            tests::should_panic_test::expected_panic_but_didnt_with_expected
            tests::should_panic_test::expected_panic_but_didnt_with_expected_multiple
            tests::should_panic_test::didnt_expect_panic
        "},
    );
}

#[test]
fn printing_in_contracts() {
    let temp = setup_package("contract_printing");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        warn: libfunc `print` is not allowed in the libfuncs list `Default libfunc list`
         --> contract: HelloStarknet
        help: try compiling with the `experimental` list
         --> Scarb.toml
            [[target.starknet-contract]]
            allowed-libfuncs-list.name = "experimental"

        [..]Finished[..]


        Collected 2 test(s) from contract_printing package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        Hello world!
        [PASS] tests::test_contract::test_increase_balance [..]
        [PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "#},
    );
}

#[test]
fn incompatible_snforge_std_version_warning() {
    let temp = setup_package("steps");
    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["dev-dependencies"]["snforge_std"]["path"] = Item::None;
    scarb_toml["dev-dependencies"]["snforge_std"]["git"] =
        value("https://github.com/foundry-rs/starknet-foundry.git");
    scarb_toml["dev-dependencies"]["snforge_std"]["tag"] = value("v0.10.1");
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Updating git repository https://github.com/foundry-rs/starknet-foundry
        [WARNING] Package snforge_std version does not meet the recommended version requirement =0.21.0, [..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from steps package
        Running 4 test(s) from src/
        [PASS] steps::tests::steps_570031 [..]
        [FAIL] steps::tests::steps_4000006
        
        Failure data:
            Could not reach the end of the program. RunResources has no remaining steps.
        
        [FAIL] steps::tests::steps_5700031
        
        Failure data:
            Could not reach the end of the program. RunResources has no remaining steps.
        
        [PASS] steps::tests::steps_3999987 [..]
        Tests: 2 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            steps::tests::steps_4000006
            steps::tests::steps_5700031
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
        [PASS] tests::test_complex::complex[..]
                steps: [..]
                memory holes: [..]
                builtins: ([..])
                syscalls: ([..])
                
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
fn catch_runtime_errors() {
    let temp = setup_package("simple_package");

    temp.child("tests/test.cairo")
        .write_str(indoc!(
            r#"
                use snforge_std::fs::{FileTrait, read_txt};

                #[test]
                #[should_panic(expected: "No such file or directory (os error 2)")]
                fn catch_no_such_file() {
                    let file = FileTrait::new("no_way_this_file_exists");
                    let content = read_txt(@file);

                    assert!(false);
                }
            "#
        ))
        .unwrap();

    let output = test_runner(&temp).assert();

    assert_stdout_contains(
        output,
        formatdoc!(
            r#"
                [..]Compiling[..]
                [..]Finished[..]
                [PASS] tests::test::catch_no_such_file [..]
            "#
        ),
    );
}
