use std::path::PathBuf;

use assert_fs::fixture::PathCopy;
use indoc::indoc;

use crate::e2e::common::runner::runner;

#[test]
fn workspace_run_without_arguments() {
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
        Collected 3 test(s) and 2 test file(s)
        Running 1 test(s) from hello_workspaces package
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
fn workspace_run_specific_package() {
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
        Collected 4 test(s) and 3 test file(s)
        Running 1 test(s) from addition package
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
fn workspace_run_specific_package_and_name() {
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
        Collected 1 test(s) and 3 test file(s)
        Running 0 test(s) from addition package
        Running 0 test(s) from tests/nested/test_nested.cairo
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 1 passed, 0 failed, 0 skipped
        "#});
}

#[test]
fn workspace_specify_root_package() {
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
        Collected 3 test(s) and 2 test file(s)
        Running 1 test(s) from hello_workspaces package
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
fn workspace_run_inside_nested_package() {
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
        Collected 4 test(s) and 3 test file(s)
        Running 1 test(s) from addition package
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
fn workspace_run_for_entire_workspace() {
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
        Collected 4 test(s) and 3 test file(s)
        Running 1 test(s) from addition package
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        Collected 1 test(s) and 1 test file(s)
        Running 1 test(s) from fibonacci package
        [PASS] fibonacci::tests::it_works
        Tests: 1 passed, 0 failed, 0 skipped
        Collected 3 test(s) and 2 test file(s)
        Running 1 test(s) from hello_workspaces package
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
fn workspace_run_missing_package() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    let snapbox = runner().arg("--package").arg("missing_package");

    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        "#});
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
        Collected 4 test(s) and 3 test file(s)
        Running 1 test(s) from addition package
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        Collected 1 test(s) and 1 test file(s)
        Running 1 test(s) from fibonacci package
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
        Collected 4 test(s) and 3 test file(s)
        Running 1 test(s) from addition package
        [PASS] addition::tests::it_works
        Running 2 test(s) from tests/nested/test_nested.cairo
        [PASS] test_nested::test_two
        [PASS] test_nested::test_two_and_two
        Running 1 test(s) from tests/test_simple.cairo
        [PASS] test_simple::simple_case
        Tests: 4 passed, 0 failed, 0 skipped
        "#});
}
