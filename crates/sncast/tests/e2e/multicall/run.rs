use crate::helpers::constants::{ACCOUNT_FILE_PATH, MULTICALL_CONFIGS_DIR, URL};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::create_and_deploy_oz_account;
use crate::helpers::runner::runner;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use std::path::Path;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case(account: &str) {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        account,
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args);
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_calldata_ids() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke_calldata_ids.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_invalid_path() {
    let tempdir = create_and_deploy_oz_account().await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        "non-existent",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stdout().is_empty());

    let expected_file_error = "No such file or directory [..]";

    assert_stderr_contains(
        output,
        formatdoc! {r"
        Command: multicall run
        Error: {}
        ", expected_file_error},
    );
}

#[tokio::test]
async fn test_deploy_fail() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invalid.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall run
        Error: Transaction execution error [..]
        "},
    );
}

#[tokio::test]
async fn test_invoke_fail() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("invoke_invalid.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall run
        Error: Transaction execution error [..]
        "},
    );
}

#[tokio::test]
async fn test_deploy_success_invoke_fails() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_succ_invoke_fail.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall run
        Error: Transaction execution error [..]
        "},
    );
}

#[tokio::test]
async fn test_numeric_inputs() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke_numeric_inputs.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_numeric_overflow() {
    let tempdir = create_and_deploy_oz_account().await;

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke_numeric_overflow.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall run
        Error: Failed to parse [..]
        number too large to fit in target type
        "},
    );
}
