use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};

use crate::e2e::common::runner::{get_current_branch, get_remote_url, runner, setup_package};
use assert_fs::TempDir;
use std::str::FromStr;

#[test]
fn simple_package() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 11 test(s) and 5 test file(s)
        Running 1 test(s) from simple_package package
        [PASS] simple_package::test_fib
        Running 1 test(s) from tests/contract.cairo
        [PASS] contract::call_and_invoke
        Running 2 test(s) from tests/ext_function_test.cairo
        [PASS] ext_function_test::test_my_test
        [PASS] ext_function_test::test_simple
        Running 6 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_simple
        [PASS] test_simple::test_simple2
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        [FAIL] test_simple::test_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        [FAIL] test_simple::test_another_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        Running 1 test(s) from tests/without_prefix.cairo
        [PASS] without_prefix::five
        Tests: 9 passed, 2 failed, 0 skipped
        "#});
}

#[test]
fn simple_package_with_git_dependency() {
    let temp = TempDir::new().unwrap();
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
            starknet = "2.2.0"
            snforge_std = {{ git = "https://github.com/{}", branch = "{}" }}
            "#,
            remote_url,
            branch
        ))
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Updating git repository[..]
        [..]Compiling[..]
        [..]Finished[..]
        Collected 11 test(s) and 5 test file(s)
        Running 1 test(s) from simple_package package
        [PASS] simple_package::test_fib
        Running 1 test(s) from tests/contract.cairo
        [PASS] contract::call_and_invoke
        Running 2 test(s) from tests/ext_function_test.cairo
        [PASS] ext_function_test::test_my_test
        [PASS] ext_function_test::test_simple
        Running 6 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_simple
        [PASS] test_simple::test_simple2
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        [FAIL] test_simple::test_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        [FAIL] test_simple::test_another_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        Running 1 test(s) from tests/without_prefix.cairo
        [PASS] without_prefix::five
        Tests: 9 passed, 2 failed, 0 skipped
        "#}).stderr_matches("");
}

#[test]
fn with_failing_scarb_build() {
    let temp = setup_package("simple_package");
    let lib_file = temp.child("src/lib.cairo");
    lib_file
        .write_str(indoc!(
            r#"
        mod hello_starknet;
        mods erc20;
    "#
        ))
        .unwrap();

    let snapbox = runner();

    let result = snapbox.current_dir(&temp).assert().failure();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(stdout.contains("Scarb build didn't succeed:"));
}

#[test]
fn with_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("two")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 2 test(s) and 5 test file(s)
        Running 0 test(s) from simple_package package
        Running 0 test(s) from tests/contract.cairo
        Running 0 test(s) from tests/ext_function_test.cairo
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        Running 0 test(s) from tests/without_prefix.cairo
        Tests: 2 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_exact_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("test_simple::test_two")
        .arg("--exact")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 1 test(s) and 5 test file(s)
        Running 0 test(s) from simple_package package
        Running 0 test(s) from tests/contract.cairo
        Running 0 test(s) from tests/ext_function_test.cairo
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_two
        Running 0 test(s) from tests/without_prefix.cairo
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_non_matching_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("qwerty")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 0 test(s) and 5 test file(s)
        Running 0 test(s) from simple_package package
        Running 0 test(s) from tests/contract.cairo
        Running 0 test(s) from tests/ext_function_test.cairo
        Running 0 test(s) from tests/test_simple.cairo
        Running 0 test(s) from tests/without_prefix.cairo
        Tests: 0 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_print() {
    let temp = setup_package("print_test");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 1 test(s) and 2 test file(s)
        Running 0 test(s) from print_test package
        Running 1 test(s) from tests/test_print.cairo
        original value: [123], converted to a string: [{]
        original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
        original value: [6381921], converted to a string: [aaa]
        original value: [12], converted to a string: []
        original value: [1234]
        original value: [123456]
        original value: [1233456789]
        original value: [123345678910]
        original value: [0], converted to a string: []
        original value: [10633823966279327296825105735305134080]
        original value: [2], converted to a string: []
        original value: [11], converted to a string: []
        original value: [1234]
        original value: [123456]
        original value: [123456789]
        original value: [12345612342]
        original value: [152]
        original value: [124], converted to a string: [|]
        original value: [149]
        original value: [439721161573], converted to a string: [false]
        [PASS] test_print::test_print
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_panic_data_decoding() {
    let temp = setup_package("panic_decoding");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 2 test file(s)
        Running 0 test(s) from panic_decoding package
        Running 4 test(s) from tests/test_panic_decoding.cairo
        [PASS] test_panic_decoding::test_simple
        [FAIL] test_panic_decoding::test_panic_decoding

        Failure data:
            original value: [123], converted to a string: [{]
            original value: [6381921], converted to a string: [aaa]
            original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
            original value: [152]
            original value: [124], converted to a string: [|]
            original value: [149]

        [FAIL] test_panic_decoding::test_panic_decoding2

        Failure data:
            original value: [128]

        [PASS] test_panic_decoding::test_simple2
        Tests: 2 passed, 2 failed, 0 skipped
        "#});
}

#[test]
fn with_exit_first() {
    let temp = setup_package("simple_package");
    let scarb_path = temp.child("Scarb.toml");
    scarb_path
        .write_str(&formatdoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [dependencies]
            starknet = "2.2.0"
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

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 11 test(s) and 5 test file(s)
        Running 1 test(s) from simple_package package
        [PASS] simple_package::test_fib
        Running 1 test(s) from tests/contract.cairo
        [PASS] contract::call_and_invoke
        Running 2 test(s) from tests/ext_function_test.cairo
        [PASS] ext_function_test::test_my_test
        [PASS] ext_function_test::test_simple
        Running 6 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_simple
        [PASS] test_simple::test_simple2
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        [FAIL] test_simple::test_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        [SKIP] test_simple::test_another_failing
        [SKIP] without_prefix::five
        Tests: 8 passed, 1 failed, 2 skipped
        "#});
}

#[test]
fn with_exit_first_flag() {
    let temp = setup_package("simple_package");
    let snapbox = runner().arg("--exit-first");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 11 test(s) and 5 test file(s)
        Running 1 test(s) from simple_package package
        [PASS] simple_package::test_fib
        Running 1 test(s) from tests/contract.cairo
        [PASS] contract::call_and_invoke
        Running 2 test(s) from tests/ext_function_test.cairo
        [PASS] ext_function_test::test_my_test
        [PASS] ext_function_test::test_simple
        Running 6 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_simple
        [PASS] test_simple::test_simple2
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        [FAIL] test_simple::test_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        [SKIP] test_simple::test_another_failing
        [SKIP] without_prefix::five
        Tests: 8 passed, 1 failed, 2 skipped
        "#});
}

#[test]
fn exit_first_flag_takes_precedence() {
    let temp = setup_package("simple_package");
    let scarb_path = temp.child("simple_package/Scarb.toml");
    scarb_path
        .write_str(indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [dependencies]
            starknet = "2.2.0"
            snforge_std = { path = "../.." }

            [[target.starknet-contract]]
            sierra = true
            casm = true
            [tool.snforge]
            exit_first = false
            "#
        ))
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("--exit-first")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 11 test(s) and 5 test file(s)
        Running 1 test(s) from simple_package package
        [PASS] simple_package::test_fib
        Running 1 test(s) from tests/contract.cairo
        [PASS] contract::call_and_invoke
        Running 2 test(s) from tests/ext_function_test.cairo
        [PASS] ext_function_test::test_my_test
        [PASS] ext_function_test::test_simple
        Running 6 test(s) from tests/test_simple.cairo
        [PASS] test_simple::test_simple
        [PASS] test_simple::test_simple2
        [PASS] test_simple::test_two
        [PASS] test_simple::test_two_and_two
        [FAIL] test_simple::test_failing

        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]

        [SKIP] test_simple::test_another_failing
        [SKIP] without_prefix::five
        Tests: 8 passed, 1 failed, 2 skipped
        "#});
}

#[test]
fn using_corelib_names() {
    let temp = setup_package("using_corelib_names");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 5 test file(s)
        Running 0 test(s) from using_corelib_names package
        Running 1 test(s) from tests/bits.cairo
        [PASS] bits::test_names
        Running 1 test(s) from tests/math.cairo
        [PASS] math::test_names
        Running 1 test(s) from tests/test.cairo
        [PASS] test::test_names
        Running 1 test(s) from tests/types.cairo
        [PASS] types::test_names
        Tests: 4 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn should_panic() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/should_panic_test", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! { r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 6 test(s) and 2 test file(s)
        Running 0 test(s) from should_panic_test package
        Running 6 test(s) from tests/should_panic_test.cairo
        [PASS] should_panic_test::should_panic_no_data

        Success data:
            original value: [0], converted to a string: []

        [PASS] should_panic_test::should_panic_check_data
        [PASS] should_panic_test::should_panic_multiple_messages
        [FAIL] should_panic_test::should_panic_with_non_matching_data

        Failure data:
            Incorrect panic data
            Actual:    [8111420071579136082810415440747] (failing check)
            Expected:  [0] ()

        [FAIL] should_panic_test::didnt_expect_panic

        Failure data:
            original value: [156092886226808350968498952598218238307], converted to a string: [unexpected panic]

        [FAIL] should_panic_test::expected_panic_but_didnt
        Tests: 3 passed, 3 failed, 0 skipped
        "#});
}

#[test]
fn printing_in_contracts() {
    let temp = setup_package("contract_printing");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        warn: libfunc `cheatcode` is not allowed in the libfuncs list `Default libfunc list`
         --> contract: HelloStarknet
        help: try compiling with the `experimental` list
         --> Scarb.toml
            [[target.starknet-contract]]
            allowed-libfuncs-list.name = "experimental"
        
        [..]Finished[..]
        Collected 2 test(s) and 2 test file(s)
        Running 0 test(s) from contract_printing package
        Running 2 test(s) from tests/test_contract.cairo
        original value: [22405534230753963835153736737], converted to a string: [Hello world!]
        [PASS] test_contract::test_increase_balance
        [PASS] test_contract::test_cannot_increase_balance_with_zero_value
        Tests: 2 passed, 0 failed, 0 skipped
        "#});
}
