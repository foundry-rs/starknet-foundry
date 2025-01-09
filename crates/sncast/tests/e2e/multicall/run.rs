use crate::helpers::constants::{ACCOUNT_FILE_PATH, MULTICALL_CONFIGS_DIR, URL};
use crate::helpers::fixtures::create_and_deploy_oz_account;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, AsOutput};
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

    let snapbox = runner(&args);
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        command: multicall run
        transaction_hash: 0x0[..]

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

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        command: multicall run
        transaction_hash: 0x0[..]

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
    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: No such file or directory [..]
        "},
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
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: Transaction execution error [..]
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
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: Transaction execution error [..]
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
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: Transaction execution error [..]
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

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_matches(indoc! {r"
        command: multicall run
        transaction_hash: 0x0[..]

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
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: Failed to parse [..]
        number too large to fit in target type
        "},
    );
}

#[tokio::test]
async fn test_version_deprecation_warning() {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "oz",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--version",
        "v3",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert();

    output.stdout_matches(indoc! {r"
        [WARNING] The '--version' flag is deprecated and will be removed in the future. Version 3 will become the only type of transaction available.
        command: multicall run
        transaction_hash: 0x0[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_version_deprecation_warning_error() {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "oz",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--version",
        "v2137",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert();

    output.stderr_matches(indoc! {r"
        error: invalid value 'v2137' for '--version <VERSION>': Invalid value 'v2137'. Possible values: v1, v3

        For more information, try '--help'.
    "});
}
