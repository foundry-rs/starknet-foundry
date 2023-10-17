use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};

use crate::assert_stdout_contains;
use crate::e2e::common::runner::{get_current_branch, get_remote_url, runner, setup_package};
use assert_fs::TempDir;
use std::{path::Path, str::FromStr};

#[test]
fn simple_package() {
    let temp = setup_package("simple_package");
    let snapbox = runner();
    let output = snapbox.current_dir(&temp).assert().code(1);

    assert_stdout_contains!(
        output,
        indoc! {r#"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 11 test(s) from simple_package package
    Running 1 test(s) from src/
    [PASS] simple_package::test_fib
    Running 10 test(s) from tests/
    [PASS] tests::contract::call_and_invoke
    [PASS] tests::ext_function_test::test_my_test
    [PASS] tests::ext_function_test::test_simple
    [PASS] tests::test_simple::test_simple
    [PASS] tests::test_simple::test_simple2
    [PASS] tests::test_simple::test_two
    [PASS] tests::test_simple::test_two_and_two
    [FAIL] tests::test_simple::test_failing
    
    Failure data:
        original value: [8111420071579136082810415440747], converted to a string: [failing check]
    
    [FAIL] tests::test_simple::test_another_failing
    
    Failure data:
        original value: [8111420071579136082810415440747], converted to a string: [failing check]
    
    [PASS] tests::without_prefix::five
    Tests: 9 passed, 2 failed, 0 skipped
    
    Failures:
        tests::test_simple::test_failing
        tests::test_simple::test_another_failing
    "#}
    );
}

#[test]
fn simple_package_with_git_dependency() {
    let temp = TempDir::new().unwrap();
    let temp_scarb = TempDir::new().unwrap();

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
    let output = snapbox
        .env("SCARB_CACHE", temp_scarb.path())
        .current_dir(&temp)
        .assert()
        .code(1);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Updating git repository[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 11 test(s) from simple_package package
        Running 1 test(s) from src/
        [PASS] simple_package::test_fib
        Running 10 test(s) from tests/
        [PASS] tests::contract::call_and_invoke
        [PASS] tests::ext_function_test::test_my_test
        [PASS] tests::ext_function_test::test_simple
        [PASS] tests::test_simple::test_simple
        [PASS] tests::test_simple::test_simple2
        [PASS] tests::test_simple::test_two
        [PASS] tests::test_simple::test_two_and_two
        [FAIL] tests::test_simple::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [FAIL] tests::test_simple::test_another_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [PASS] tests::without_prefix::five
        Tests: 9 passed, 2 failed, 0 skipped
        
        Failures:
            tests::test_simple::test_failing
            tests::test_simple::test_another_failing
        "#}
    );
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

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(stdout.contains("Scarb build did not succeed"));
}

#[test]
fn with_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).arg("two").assert().success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::test_simple::test_two
        [PASS] tests::test_simple::test_two_and_two
        Tests: 2 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn with_filter_matching_module() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    let output = snapbox
        .current_dir(&temp)
        .arg("ext_function_test::")
        .assert()
        .success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 2 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::ext_function_test::test_my_test
        [PASS] tests::ext_function_test::test_simple
        Tests: 2 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn with_exact_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    let output = snapbox
        .current_dir(&temp)
        .arg("tests::test_simple::test_two")
        .arg("--exact")
        .assert()
        .success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] tests::test_simple::test_two
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn with_non_matching_filter() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).arg("qwerty").assert().success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 0 test(s) from simple_package package
        Running 0 test(s) from src/
        Running 0 test(s) from tests/
        Tests: 0 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn with_print() {
    let temp = setup_package("print_test");
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().success();

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from print_test package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        original value: [123], converted to a string: [{]
        original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
        original value: [6381921], converted to a string: [aaa]
        original value: [12]
        original value: [1234]
        original value: [123456]
        original value: [1233456789]
        original value: [123345678910]
        original value: [0]
        original value: [10633823966279327296825105735305134080]
        original value: [2]
        original value: [11]
        original value: [1234]
        original value: [123456]
        original value: [123456789]
        original value: [12345612342]
        original value: [152]
        original value: [124], converted to a string: [|]
        original value: [149]
        original value: [439721161573], converted to a string: [false]
        original value: [27]
        original value: [17]
        original value: [37], converted to a string: [%]
        original value: [127]
        original value: [32], converted to a string: [ ]
        original value: [166906514068638843492736773029576256], converted to a string: [ % abc 123 !?>@]
        [PASS] tests::test_print::test_print
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}

#[test]
fn with_panic_data_decoding() {
    let temp = setup_package("panic_decoding");
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().code(1);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from panic_decoding package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [PASS] tests::test_panic_decoding::test_simple
        [FAIL] tests::test_panic_decoding::test_panic_decoding
        
        Failure data:
            original value: [123], converted to a string: [{]
            original value: [6381921], converted to a string: [aaa]
            original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
            original value: [152]
            original value: [124], converted to a string: [|]
            original value: [149]
        
        [FAIL] tests::test_panic_decoding::test_panic_decoding2
        
        Failure data:
            original value: [128]
        
        [PASS] tests::test_panic_decoding::test_simple2
        Tests: 2 passed, 2 failed, 0 skipped
        
        Failures:
            tests::test_panic_decoding::test_panic_decoding
            tests::test_panic_decoding::test_panic_decoding2
        "#}
    );
}

#[test]
#[ignore = "Non deterministic"]
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

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from exit_first package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [SKIP] tests::ext_function_test::hard_test
        [SKIP] tests::ext_function_test::hard_test1
        [FAIL] tests::ext_function_test::simple_test

        Failure data:
            original value: [35718230152306872753561363307], converted to a string: [simple check]

        [SKIP] tests::ext_function_test::hard_test2
        Tests: 0 passed, 1 failed, 3 skipped

        Failures:
            tests::ext_function_test::simple_test
        "#}
    );
}

#[test]
#[ignore = "Non deterministic"]
fn with_exit_first_flag() {
    let temp = setup_package("exit_first");
    let snapbox = runner().arg("--exit-first");

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from exit_first package
        Running 0 test(s) from src/
        Running 4 test(s) from tests/
        [SKIP] tests::ext_function_test::hard_test
        [SKIP] tests::ext_function_test::hard_test1
        [FAIL] tests::ext_function_test::simple_test

        Failure data:
            original value: [35718230152306872753561363307], converted to a string: [simple check]

        [SKIP] tests::ext_function_test::hard_test2
        Tests: 0 passed, 1 failed, 3 skipped

        Failures:
            tests::ext_function_test::simple_test
        "#}
    );
}

#[test]
fn init_new_project_test() {
    let temp = TempDir::new().unwrap();
    let temp_scarb = TempDir::new().unwrap();

    let snapbox = runner();
    snapbox
        .env("SCARB_CACHE", temp_scarb.path())
        .current_dir(&temp)
        .arg("--init")
        .arg("test_name")
        .assert()
        .success();
    let manifest_path = temp.child("test_name/Scarb.toml");

    let generated_toml = std::fs::read_to_string(manifest_path.path()).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let expected_toml = formatdoc!(
        r#"
            [package]
            name = "test_name"
            version = "0.1.0"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            snforge_std = {{ git = "https://github.com/foundry-rs/starknet-foundry", tag = "v{}" }}
            starknet = "2.2.0"

            [[target.starknet-contract]]
            casm = true
            # foo = {{ path = "vendor/foo" }}
        "#,
        version
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
        starknet = "2.2.0"
        snforge_std = {{ git = "https://github.com/{}", branch = "{}" }}
        "#,
            remote_url,
            branch
        ))
        .unwrap();

    let snapbox = runner();
    // Check if template works with current version of snforge_std
    let output = snapbox
        .current_dir(temp.child(Path::new("test_name")))
        .assert()
        .success();
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Updating git repository[..]
        [..]Compiling test_name v0.1.0[..]
        [..]Finished[..]


        Collected 2 test(s) from test_name package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::test_contract::test_increase_balance
        [PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value
        Tests: 2 passed, 0 failed, 0 skipped
    "#}
    );
}

#[test]
fn should_panic() {
    let temp = TempDir::new().unwrap();
    temp.copy_from("tests/data/should_panic_test", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains!(
        output,
        indoc! { r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 6 test(s) from should_panic_test package
        Running 0 test(s) from src/
        Running 6 test(s) from tests/
        [PASS] tests::should_panic_test::should_panic_no_data

        Success data:
            original value: [0], converted to a string: []

        [PASS] tests::should_panic_test::should_panic_check_data
        [PASS] tests::should_panic_test::should_panic_multiple_messages
        [FAIL] tests::should_panic_test::should_panic_with_non_matching_data

        Failure data:
            Incorrect panic data
            Actual:    [8111420071579136082810415440747] (failing check)
            Expected:  [0] ()

        [FAIL] tests::should_panic_test::didnt_expect_panic

        Failure data:
            original value: [156092886226808350968498952598218238307], converted to a string: [unexpected panic]

        [FAIL] tests::should_panic_test::expected_panic_but_didnt
        Tests: 3 passed, 3 failed, 0 skipped

        Failures:
            tests::should_panic_test::should_panic_with_non_matching_data
            tests::should_panic_test::didnt_expect_panic
            tests::should_panic_test::expected_panic_but_didnt
        "#}
    );
}

#[test]
fn printing_in_contracts() {
    let temp = setup_package("contract_printing");
    let snapbox = runner();

    let output = snapbox.current_dir(&temp).assert().success();
    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Compiling[..]
        warn: libfunc `cheatcode` is not allowed in the libfuncs list `Default libfunc list`
         --> contract: HelloStarknet
        help: try compiling with the `experimental` list
         --> Scarb.toml
            [[target.starknet-contract]]
            allowed-libfuncs-list.name = "experimental"

        [..]Finished[..]


        Collected 2 test(s) from contract_printing package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        original value: [22405534230753963835153736737], converted to a string: [Hello world!]
        [PASS] tests::test_contract::test_increase_balance
        [PASS] tests::test_contract::test_cannot_increase_balance_with_zero_value
        Tests: 2 passed, 0 failed, 0 skipped
        "#}
    );
}
