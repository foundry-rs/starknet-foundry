use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::common::runner::runner;

#[test]
fn simple_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_test", &["**/*"]).unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 9 test(s) and 4 test file(s)
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 5 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            Running 1 test(s) from tests/without_prefix.cairo
            [PASS] without_prefix::without_prefix::five
            Tests: 8 passed, 1 failed, 0 skipped
        "#});
}

#[test]
fn with_filter() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_test", &["**/*"]).unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("two")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 2 test(s) and 4 test file(s)
            Running 0 test(s) from src/lib.cairo
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
    temp.copy_from("tests/data/simple_test", &["**/*"]).unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("test_simple::test_simple::test_two")
        .arg("--exact")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 1 test(s) and 4 test file(s)
            Running 0 test(s) from src/lib.cairo
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
    temp.copy_from("tests/data/simple_test", &["**/*"]).unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("qwerty")
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 0 test(s) and 4 test file(s)
            Running 0 test(s) from src/lib.cairo
            Running 0 test(s) from tests/ext_function_test.cairo
            Running 0 test(s) from tests/test_simple.cairo
            Running 0 test(s) from tests/without_prefix.cairo
            Tests: 0 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn with_declare() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/declare_test", &["**/*"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 3 test(s) and 3 test file(s)
            Running 0 test(s) from src/contract1.cairo
            Running 0 test(s) from src/lib.cairo
            Running 3 test(s) from tests/test_declare.cairo
            [PASS] test_declare::test_declare::test_declare_simple
            [PASS] test_declare::test_declare::multiple_contracts
            [FAIL] test_declare::test_declare::non_existent_contract

            Failure data:
                Got an exception while executing a hint:
                Failed to find contract GoodbyeStarknet in starknet_artifacts.json

            Tests: 2 passed, 1 failed, 0 skipped
        "#});
}

#[test]
fn run_with_multiple_contracts() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/multicontract", &["**/*"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 6 test(s) and 5 test file(s)
            Running 0 test(s) from src/contract1.cairo
            Running 0 test(s) from src/contract2.cairo
            Running 1 test(s) from src/lib.cairo
            [PASS] [..]src::test_fib
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 3 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            Tests: 5 passed, 1 failed, 0 skipped
        "#});
}

#[test]
fn with_print() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/print_test", &["**/*"]).unwrap();

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
fn panic_data_decoding() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/panic_decoding_test", &["**/*"])
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
fn exit_first() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/exit_first_test", &["**/*"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 10 test(s) and 4 test file(s)
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 6 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [FAIL] test_simple::test_simple::test_early_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            [SKIP] test_simple::test_simple::test_two
            [SKIP] test_simple::test_simple::test_two_and_two
            [SKIP] test_simple::test_simple::test_failing
            [SKIP] without_prefix::without_prefix::five
            Tests: 5 passed, 1 failed, 4 skipped
        "#});
}

#[test]
fn exit_first_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_test", &["**/*"]).unwrap();

    let snapbox = runner().arg("--exit-first");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 9 test(s) and 4 test file(s)
            Running 1 test(s) from src/lib.cairo
            [PASS] src::test_fib
            Running 2 test(s) from tests/ext_function_test.cairo
            [PASS] ext_function_test::ext_function_test::test_my_test
            [PASS] ext_function_test::ext_function_test::test_simple
            Running 5 test(s) from tests/test_simple.cairo
            [PASS] test_simple::test_simple::test_simple
            [PASS] test_simple::test_simple::test_simple2
            [PASS] test_simple::test_simple::test_two
            [PASS] test_simple::test_simple::test_two_and_two
            [FAIL] test_simple::test_simple::test_failing

            Failure data:
                original value: [8111420071579136082810415440747], converted to a string: [failing check]

            [SKIP] without_prefix::without_prefix::five
            Tests: 7 passed, 1 failed, 1 skipped
        "#});
}

#[test]
fn dispatchers() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/dispatchers", &["**/*"]).unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"Collected 2 test(s) and 4 test file(s)
            Running 0 test(s) from src/erc20.cairo
            Running 0 test(s) from src/hello_starknet.cairo
            Running 0 test(s) from src/lib.cairo
            Running 2 test(s) from tests/using_dispatchers.cairo
            [PASS] using_dispatchers::using_dispatchers::call_and_invoke
            [PASS] using_dispatchers::using_dispatchers::advanced_types
            Tests: 2 passed, 0 failed, 0 skipped
        "#});
}
