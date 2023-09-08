use std::path::PathBuf;

use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::e2e::common::runner::runner;

#[test]
fn run_without_arguments() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
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
        "#});
}

#[test]
fn run_specific_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("addition");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn run_specific_package2() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("fibonacci");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 1 test(s) and 1 test file(s) from fibonacci package
        Running 1 inline test(s)
        [PASS] fibonacci::tests::it_works
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}
#[test]
fn run_specific_package_and_name() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("simple").arg("--package").arg("addition");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 1 test(s) and 3 test file(s) from addition package
        Running 0 inline test(s)
        Running 0 test(s) from tests/nested/test_nested.cairo
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn run_specify_root_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("hello_workspaces");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
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
        "#});
}

#[test]
fn run_inside_nested_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let package_dir = temp.join(PathBuf::from("crates/addition"));

    let snapbox = runner();

    snapbox
        .current_dir(package_dir)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn run_for_entire_workspace() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--workspace");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped


        Collected 1 test(s) and 1 test file(s) from fibonacci package
        Running 1 inline test(s)
        [PASS] fibonacci::tests::it_works
        Tests: 1 passed, 0 failed, 0 skipped


        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        "#});
}

#[test]
fn run_for_entire_workspace_inside_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let package_dir = temp.join(PathBuf::from("crates/fibonacci"));

    let snapbox = runner().arg("--workspace");

    snapbox
        .current_dir(&package_dir)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped


        Collected 1 test(s) and 1 test file(s) from fibonacci package
        Running 1 inline test(s)
        [PASS] fibonacci::tests::it_works
        Tests: 1 passed, 0 failed, 0 skipped


        Collected 3 test(s) and 2 test file(s) from hello_workspaces package
        Running 1 inline test(s)
        [PASS] hello_workspaces::test_simple
        Running 2 test(s) from tests/test_failing.cairo
        [FAIL] test_failing::test_failing
        
        Failure data:
            original value: [8111420071579136082810415440747], converted to a string: [failing check]
        
        [SKIP] test_failing::test_another_failing
        Tests: 1 passed, 1 failed, 1 skipped
        "#});
}

#[test]
fn run_for_entire_workspace_and_specific_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--workspace").arg("--package").arg("addition");

    let result = snapbox.current_dir(&temp).assert().failure();

    let stderr = String::from_utf8_lossy(&result.get_output().stderr);

    assert!(stderr.contains("the argument '--workspace' cannot be used with '--package <SPEC>'"));
}

#[test]
fn run_missing_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("missing_package");

    let result = snapbox.current_dir(&temp).assert().failure();

    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    assert!(stdout.contains("Failed to find any packages matching the specified filter"));
}

#[test]
fn virtual_workspace() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/virtual_workspace", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped


        Collected 1 test(s) and 1 test file(s) from fibonacci package
        Running 1 inline test(s)
        [PASS] fibonacci::tests::it_works
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn virtual_workspace_specify_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/virtual_workspace", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("addition");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Compiling[..]
        [..]Finished[..]
        Collected 4 test(s) and 3 test file(s) from addition package
        Running 1 inline test(s)
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        "#});
}
