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


        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/
        [FAIL] tests::test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] tests::test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            tests::test_failing::test_failing
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


        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
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


        Collected 7 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        
        Failures:
            tests::abc::efg::failing_test
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


        Collected 1 test(s) from addition package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] tests::nested::simple_case
        Tests: 1 passed, 0 failed, 0 skipped
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


        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/
        [FAIL] tests::test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] tests::test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            tests::test_failing::test_failing
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


        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
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
        
        
        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
        Tests: 5 passed, 0 failed, 0 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 7 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/
        [FAIL] tests::test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] tests::test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            tests::abc::efg::failing_test
            tests::test_failing::test_failing
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
        
        
        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
        Tests: 5 passed, 0 failed, 0 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 7 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works
        [PASS] fibonacci::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/
        [FAIL] tests::test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] tests::test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        
        Failures:
            tests::abc::efg::failing_test
            tests::test_failing::test_failing
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
        
        
        Collected 7 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            tests::abc::efg::failing_test
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


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
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
        
        
        Collected 7 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        
        Failures:
            tests::abc::efg::failing_test
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


        Collected 1 test(s) from subtraction package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] tests::nested::simple_case
        Tests: 1 passed, 0 failed, 0 skipped
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


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
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
        
        
        Collected 7 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            tests::abc::efg::failing_test
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
        
        
        Collected 7 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works
        [PASS] fibonacci2::tests::contract_test
        Running 5 test(s) from tests/
        [PASS] tests::lib_test
        [PASS] tests::abc::abc_test
        [PASS] tests::abc::efg::efg_test
        [FAIL] tests::abc::efg::failing_test
        
        Failure data:
            original value: [0], converted to a string: []
        
        [SKIP] tests::abc::efg::skipped_test
        Tests: 5 passed, 1 failed, 1 skipped
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works
        Running 4 test(s) from tests/
        [PASS] tests::nested::simple_case
        [PASS] tests::nested::contract_test
        [PASS] tests::nested::test_nested::test_two
        [PASS] tests::nested::test_nested::test_two_and_two
        Tests: 5 passed, 0 failed, 0 skipped
        
        Failures:
            tests::abc::efg::failing_test
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
