use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;

#[test]
fn with_one_core() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("--cores")
        .arg("1")
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
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
        "#});
}

#[test]
fn with_more_than_one_core() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("--cores")
        .arg("3")
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
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
        "#});
}

#[test]
fn with_too_much_cores() {
    let temp = setup_package("simple_package");
    let snapbox = runner();

    let assert = snapbox
        .current_dir(&temp)
        .arg("--cores")
        .arg("99999")
        .assert()
        .code(2);

    let stderr =
        String::from_utf8(assert.get_output().stderr.clone()).expect("stderr is not valid UTF-8");

    assert!(stderr.contains("error: invalid value '99999' for '--cores <CORES>': Number of cores must be less than or equal to the number of cores available on the machine"));

    let re = regex::Regex::new(r"\(\d+\)").expect("Invalid regex");
    assert!(
        re.is_match(&stderr),
        "stderr does not contain the expected pattern"
    );
}
