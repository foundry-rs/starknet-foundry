use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use indoc::indoc;

use crate::e2e::common::runner::runner;

#[test]
fn simple_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 11 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 1 test(s) from tests/contract.cairo
            [PASS] contract::contract::call_and_invoke
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 6 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing
            
            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]
            
            [FAIL] test_simple::test_simple::test_another_failing
            
            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]
            
            Running 1 test(s) from tests/without_prefix.cairo
            [PASS] without_prefix::without_prefix::five
            Tests: 9 passed, 2 failed, 0 skipped
            "#});
}

#[test]
fn with_filter() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("two")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 2 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 0 test(s) from src/lib.cairo
            Running 0 test(s) from tests/contract.cairo
            Running 0 test(s) from tests/ext_function_test.cairo
            Running 2 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            Running 0 test(s) from tests/without_prefix.cairo
            Tests: 2 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_exact_filter() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("test_simple::test_simple::test_two")
        .arg("--exact")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 1 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 0 test(s) from src/lib.cairo
            Running 0 test(s) from tests/contract.cairo
            Running 0 test(s) from tests/ext_function_test.cairo
            Running 1 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_two
            Running 0 test(s) from tests/without_prefix.cairo
            Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_non_matching_filter() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("qwerty")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 0 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 0 test(s) from src/lib.cairo
            Running 0 test(s) from tests/contract.cairo
            Running 0 test(s) from tests/ext_function_test.cairo
            Running 0 test(s) from tests/test_simple.cairo
            Running 0 test(s) from tests/without_prefix.cairo
            Tests: 0 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_print() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/print_test", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 1 test(s) and 2 test file(s)
            Running 0 test(s) from src/lib.cairo
            Running 1 test(s) from tests/test_print.cairo
            original value: [123], converted to a string: [{]
            original value: [6381921], converted to a string: [aaa]
            original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
            original value: [152]
            original value: [124], converted to a string: [|]
            original value: [149]
            original value: [439721161573], converted to a string: [false]
            [PASS] test_print::test_print::test_print
            Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_panic_data_decoding() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from(
        "tests/data/panic_decoding_test",
        &["**/*.cairo", "**/*.toml"],
    )
    .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 4 test(s) and 2 test file(s)
            Running 0 test(s) from src/lib.cairo
            Running 4 test(s) from tests/test_panic_decoding.cairo
            [PASS] test_panic_decoding::test_panic_decoding::test_simple
            [FAIL] test_panic_decoding::test_panic_decoding::test_panic_decoding

            Failure data:
                original value: [123], converted to a string: [{]
                original value: [6381921], converted to a string: [aaa]
                original value: [3618502788666131213697322783095070105623107215331596699973092056135872020480]
                original value: [152]
                original value: [124], converted to a string: [|]
                original value: [149]

            [FAIL] test_panic_decoding::test_panic_decoding::test_panic_decoding2

            Failure data:
                original value: [128]

            [PASS] test_panic_decoding::test_panic_decoding::test_simple2
            Tests: 2 passed, 2 failed, 0 skipped
        "#});
}

#[test]
fn with_exit_first() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();
    let scarb_path = temp.child("Scarb.toml");
    scarb_path
        .write_str(indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"
            
            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest
            
            [dependencies]
            starknet = "2.0.1"
            
            [[target.starknet-contract]]
            sierra = true
            casm = true
            [tool.snforge]
            exit_first = true
            "#
        ))
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 11 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 1 test(s) from tests/contract.cairo
            [PASS] contract::contract::call_and_invoke
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 6 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            [SKIP] test_simple::test_simple::test_another_failing
            [SKIP] without_prefix::without_prefix::five
            Tests: 8 passed, 1 failed, 2 skipped
        "#});
}

#[test]
fn with_exit_first_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--exit-first");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 11 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 1 test(s) from tests/contract.cairo
            [PASS] contract::contract::call_and_invoke
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 6 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            [SKIP] test_simple::test_simple::test_another_failing
            [SKIP] without_prefix::without_prefix::five
            Tests: 8 passed, 1 failed, 2 skipped
        "#});
}

#[test]
fn exit_first_flag_takes_precedence() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();
    let scarb_path = temp.child("Scarb.toml");
    scarb_path
        .write_str(indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest

            [dependencies]
            starknet = "2.0.1"

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
        .stdout_matches(indoc! {r#"Collected 11 test(s) and 7 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 1 test(s) from tests/contract.cairo
            [PASS] contract::contract::call_and_invoke
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 6 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            [SKIP] test_simple::test_simple::test_another_failing
            [SKIP] without_prefix::without_prefix::five
            Tests: 8 passed, 1 failed, 2 skipped
        "#});
}
