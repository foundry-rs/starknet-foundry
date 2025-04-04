use super::common::runner::{setup_hello_workspace, setup_virtual_workspace, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::path::PathBuf;

#[test]
fn root_workspace_without_arguments() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::tests::test_simple [..]
        Running 2 test(s) from tests/
        [FAIL] hello_workspaces_integrationtest::test_failing::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] hello_workspaces_integrationtest::test_failing::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        Tests: 1 passed, 2 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            hello_workspaces_integrationtest::test_failing::test_failing
            hello_workspaces_integrationtest::test_failing::test_another_failing
        "},
    );
}

#[test]
fn root_workspace_specific_package() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .args(["--package", "addition"])
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] addition_integrationtest::nested::simple_case [..]
        [PASS] addition_integrationtest::nested::contract_test [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}

#[test]
fn root_workspace_specific_package2() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .args(["--package", "fibonacci"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 6 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works [..]
        [PASS] fibonacci::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci_tests::lib_test [..]
        [PASS] fibonacci_tests::abc::abc_test [..]
        [PASS] fibonacci_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci_tests::abc::efg::failing_test
        "},
    );
}

#[test]
fn root_workspace_specific_package_and_name() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .args(["simple", "--package", "addition"])
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from addition package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] addition_integrationtest::nested::simple_case [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 4 filtered out
        "},
    );
}

#[test]
fn root_workspace_specify_root_package() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .args(["--package", "hello_workspaces"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::tests::test_simple [..]
        Running 2 test(s) from tests/
        [FAIL] hello_workspaces_integrationtest::test_failing::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] hello_workspaces_integrationtest::test_failing::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        Tests: 1 passed, 2 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            hello_workspaces_integrationtest::test_failing::test_failing
            hello_workspaces_integrationtest::test_failing::test_another_failing
        "},
    );
}

#[test]
fn root_workspace_inside_nested_package() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .current_dir(temp.join("crates/addition"))
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] addition_integrationtest::nested::simple_case [..]
        [PASS] addition_integrationtest::nested::contract_test [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}

#[test]
fn root_workspace_for_entire_workspace() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp).arg("--workspace").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] addition_integrationtest::nested::simple_case [..]
        [PASS] addition_integrationtest::nested::contract_test [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        
        Collected 6 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works [..]
        [PASS] fibonacci::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci_tests::lib_test [..]
        [PASS] fibonacci_tests::abc::abc_test [..]
        [PASS] fibonacci_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        
        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::tests::test_simple [..]
        Running 2 test(s) from tests/
        [FAIL] hello_workspaces_integrationtest::test_failing::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] hello_workspaces_integrationtest::test_failing::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        Tests: 1 passed, 2 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci_tests::abc::efg::failing_test
            hello_workspaces_integrationtest::test_failing::test_failing
            hello_workspaces_integrationtest::test_failing::test_another_failing
        "},
    );
}

#[test]
fn root_workspace_for_entire_workspace_inside_package() {
    let temp = setup_hello_workspace();

    let output = test_runner(&temp)
        .current_dir(temp.join("crates/fibonacci"))
        .arg("--workspace")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) from addition package
        Running 1 test(s) from src/
        [PASS] addition::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] addition_integrationtest::nested::simple_case [..]
        [PASS] addition_integrationtest::nested::contract_test [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two [..]
        [PASS] addition_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        
        Collected 6 test(s) from fibonacci package
        Running 2 test(s) from src/
        [PASS] fibonacci::tests::it_works [..]
        [PASS] fibonacci::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci_tests::lib_test [..]
        [PASS] fibonacci_tests::abc::abc_test [..]
        [PASS] fibonacci_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        
        Collected 3 test(s) from hello_workspaces package
        Running 1 test(s) from src/
        [PASS] hello_workspaces::tests::test_simple [..]
        Running 2 test(s) from tests/
        [FAIL] hello_workspaces_integrationtest::test_failing::test_failing
        
        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        [FAIL] hello_workspaces_integrationtest::test_failing::test_another_failing

        Failure data:
            0x6661696c696e6720636865636b ('failing check')
        
        Tests: 1 passed, 2 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci_tests::abc::efg::failing_test
            hello_workspaces_integrationtest::test_failing::test_failing
            hello_workspaces_integrationtest::test_failing::test_another_failing
        "},
    );
}

#[test]
fn root_workspace_for_entire_workspace_and_specific_package() {
    let temp = setup_hello_workspace();

    let result = test_runner(&temp)
        .args(["--workspace", "--package", "addition"])
        .assert()
        .code(2);

    let stderr = String::from_utf8_lossy(&result.get_output().stderr);

    assert!(stderr.contains("the argument '--workspace' cannot be used with '--package <SPEC>'"));
}

#[test]
fn root_workspace_missing_package() {
    let temp = setup_hello_workspace();

    let result = test_runner(&temp)
        .args(["--package", "missing_package"])
        .assert()
        .code(2);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(stdout.contains("Failed to find any packages matching the specified filter"));
}

#[test]
fn virtual_workspace_without_arguments() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp);

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 6 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works [..]
        [PASS] fibonacci2::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci2_tests::lib_test [..]
        [PASS] fibonacci2_tests::abc::abc_test [..]
        [PASS] fibonacci2_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci2_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        [PASS] subtraction_integrationtest::nested::contract_test [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci2_tests::abc::efg::failing_test
        "},
    );
}

#[test]
fn virtual_workspace_specify_package() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp).arg("--package").arg("subtraction");

    let output = snapbox.current_dir(&temp).assert().success();
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        [PASS] subtraction_integrationtest::nested::contract_test [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}

#[test]
fn virtual_workspace_specific_package2() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp).arg("--package").arg("fibonacci2");

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 6 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works [..]
        [PASS] fibonacci2::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci2_tests::lib_test [..]
        [PASS] fibonacci2_tests::abc::abc_test [..]
        [PASS] fibonacci2_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci2_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci2_tests::abc::efg::failing_test
        "},
    );
}

#[test]
fn virtual_workspace_specific_package_and_name() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp)
        .arg("simple")
        .arg("--package")
        .arg("subtraction");

    let output = snapbox.current_dir(&temp).assert().success();
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from subtraction package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 4 filtered out
        "},
    );
}

#[test]
fn virtual_workspace_inside_nested_package() {
    let temp = setup_virtual_workspace();
    let package_dir = temp.join(PathBuf::from("dummy_name/subtraction"));

    let snapbox = test_runner(&temp);

    let output = snapbox.current_dir(package_dir).assert().success();
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        [PASS] subtraction_integrationtest::nested::contract_test [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        "},
    );
}

#[test]
fn virtual_workspace_for_entire_workspace() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp).arg("--workspace");

    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 6 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works [..]
        [PASS] fibonacci2::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci2_tests::lib_test [..]
        [PASS] fibonacci2_tests::abc::abc_test [..]
        [PASS] fibonacci2_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci2_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        [PASS] subtraction_integrationtest::nested::contract_test [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci2_tests::abc::efg::failing_test
        "},
    );
}

#[test]
fn virtual_workspace_for_entire_workspace_inside_package() {
    let temp = setup_virtual_workspace();
    let package_dir = temp.join(PathBuf::from("dummy_name/fibonacci2"));

    let snapbox = test_runner(&temp).arg("--workspace");
    let output = snapbox.current_dir(package_dir).assert().code(1);
    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 6 test(s) from fibonacci2 package
        Running 2 test(s) from src/
        [PASS] fibonacci2::tests::it_works [..]
        [PASS] fibonacci2::tests::contract_test [..]
        Running 4 test(s) from tests/
        [PASS] fibonacci2_tests::lib_test [..]
        [PASS] fibonacci2_tests::abc::abc_test [..]
        [PASS] fibonacci2_tests::abc::efg::efg_test [..]
        [FAIL] fibonacci2_tests::abc::efg::failing_test
        
        Failure data:
            0x0 ('')
        
        Tests: 5 passed, 1 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out


        Collected 5 test(s) from subtraction package
        Running 1 test(s) from src/
        [PASS] subtraction::tests::it_works [..]
        Running 4 test(s) from tests/
        [PASS] subtraction_integrationtest::nested::simple_case [..]
        [PASS] subtraction_integrationtest::nested::contract_test [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two [..]
        [PASS] subtraction_integrationtest::nested::test_nested::test_two_and_two [..]
        Tests: 5 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
        
        Failures:
            fibonacci2_tests::abc::efg::failing_test
        "},
    );
}

#[test]
fn virtual_workspace_for_entire_workspace_and_specific_package() {
    let temp = setup_virtual_workspace();
    let snapbox = test_runner(&temp)
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
    let snapbox = test_runner(&temp).arg("--package").arg("missing_package");

    let result = snapbox.current_dir(&temp).assert().code(2);

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(stdout.contains("Failed to find any packages matching the specified filter"));
}
