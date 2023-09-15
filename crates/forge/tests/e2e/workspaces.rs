use std::path::PathBuf;

use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_hello_workspace, setup_virtual_workspace};

#[test]
fn root_workspace_without_arguments() {
    let temp = setup_hello_workspace();

    let snapbox = runner();
    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            test_failing::test_failing
        "#});
}

#[test]
fn root_workspace_specific_package() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--package").arg("addition");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn root_workspace_specific_package2() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--package").arg("fibonacci");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) and 1 test file(s) from fibonacci package
        Running 4 inline test(s)
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        [FAIL] fibonacci::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        
        Failures:
            fibonacci::tests::failing_test
        "#});
}

#[test]
fn root_workspace_specific_package_and_name() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("simple").arg("--package").arg("addition");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) and 3 test file(s) from addition package
        Running 0 inline test(s)
        Running 0 test(s) from tests/nested/test_nested.cairo
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 2 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn root_workspace_specify_root_package() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--package").arg("hello_workspaces");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            test_failing::test_failing
        "#});
}

#[test]
fn root_workspace_inside_nested_package() {
    let temp = setup_hello_workspace();
    let package_dir = temp.join(PathBuf::from("crates/addition"));

    let snapbox = runner();

    snapbox
        .current_dir(package_dir)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn root_workspace_for_entire_workspace() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--workspace");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci package
        Running 4 inline test(s)
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        [FAIL] fibonacci::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            fibonacci::tests::failing_test
            test_failing::test_failing
        "#});
}

#[test]
fn root_workspace_for_entire_workspace_inside_package() {
    let temp = setup_hello_workspace();
    let package_dir = temp.join(PathBuf::from("crates/fibonacci"));

    let snapbox = runner().arg("--workspace");
    snapbox
        .current_dir(package_dir)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci package
        Running 4 inline test(s)
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        [FAIL] fibonacci::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            fibonacci::tests::failing_test
            test_failing::test_failing
        "#});
}

#[test]
fn root_workspace_for_entire_workspace_and_specific_package() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--workspace").arg("--package").arg("addition");

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stderr = String::from_utf8_lossy(&result.get_output().stderr);

    assert!(stderr.contains("the argument '--workspace' cannot be used with '--package <SPEC>'"));
}

#[test]
fn root_workspace_missing_package() {
    let temp = setup_hello_workspace();
    let snapbox = runner().arg("--package").arg("missing_package");

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(stdout.contains("Failed to find any packages matching the specified filter"));
}

#[test]
fn virtual_workspace_without_arguments() {
    let temp = setup_virtual_workspace();
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci2 package
        Running 4 inline test(s)
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        [FAIL] fibonacci2::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci2::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from subtraction package
        Running 1 inline test(s)
        [PASS] subtraction::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            fibonacci2::tests::failing_test
        "#});
}

#[test]
fn virtual_workspace_specify_package() {
    let temp = setup_virtual_workspace();
    let snapbox = runner().arg("--package").arg("subtraction");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from subtraction package
        Running 1 inline test(s)
        [PASS] subtraction::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn virtual_workspace_specific_package2() {
    let temp = setup_virtual_workspace();
    let snapbox = runner().arg("--package").arg("fibonacci2");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci2 package
        Running 4 inline test(s)
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        [FAIL] fibonacci2::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci2::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        
        Failures:
            fibonacci2::tests::failing_test
        "#});
}

#[test]
fn virtual_workspace_specific_package_and_name() {
    let temp = setup_virtual_workspace();
    let snapbox = runner().arg("simple").arg("--package").arg("subtraction");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) and 3 test file(s) from subtraction package
        Running 0 inline test(s)
        Running 0 test(s) from tests/nested/test_nested.cairo
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 2 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn virtual_workspace_inside_nested_package() {
    let temp = setup_virtual_workspace();
    let package_dir = temp.join(PathBuf::from("dummy_name/subtraction"));

    let snapbox = runner();

    snapbox
        .current_dir(package_dir)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from subtraction package
        Running 1 inline test(s)
        [PASS] subtraction::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn virtual_workspace_for_entire_workspace() {
    let temp = setup_virtual_workspace();
    let snapbox = runner().arg("--workspace");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci2 package
        Running 4 inline test(s)
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        [FAIL] fibonacci2::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci2::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from subtraction package
        Running 1 inline test(s)
        [PASS] subtraction::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            fibonacci2::tests::failing_test
        "#});
}

#[test]
fn virtual_workspace_for_entire_workspace_inside_package() {
    let temp = setup_virtual_workspace();
    let package_dir = temp.join(PathBuf::from("dummy_name/fibonacci2"));

    let snapbox = runner().arg("--workspace");
    snapbox
        .current_dir(package_dir)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 4 test(s) and 1 test file(s) from fibonacci2 package
        Running 4 inline test(s)
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        [FAIL] fibonacci2::tests::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] fibonacci2::tests::skipped_test
        Tests: 2 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) and 3 test file(s) from subtraction package
        Running 1 inline test(s)
        [PASS] subtraction::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 2 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        [PASS] test_simple::contract_test
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            fibonacci2::tests::failing_test
        "#});
}

#[test]
fn virtual_workspace_for_entire_workspace_and_specific_package() {
    let temp = setup_virtual_workspace();
    let snapbox = runner()
        .arg("--workspace")
        .arg("--package")
        .arg("subtraction");

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stderr = String::from_utf8_lossy(&result.get_output().stderr);

    assert!(stderr.contains("the argument '--workspace' cannot be used with '--package <SPEC>'"));
}

#[test]
fn virtual_workspace_missing_package() {
    let temp = setup_virtual_workspace();
    let snapbox = runner().arg("--package").arg("missing_package");

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(stdout.contains("Failed to find any packages matching the specified filter"));
}
