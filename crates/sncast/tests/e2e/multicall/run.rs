use crate::helpers::constants::{ACCOUNT_FILE_PATH, MULTICALL_CONFIGS_DIR, URL};
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
        "--fee-token",
        "eth",
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
        transaction_hash: 0x[..]
    "});
}

#[tokio::test]
async fn test_calldata_ids() {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invoke_calldata_ids.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user5",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--fee-token",
        "eth",
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
        transaction_hash: 0x[..]
    "});
}

#[tokio::test]
async fn test_invalid_path() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user2",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        "non-existent",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
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
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_invalid.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user2",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: An error occurred in the called contract [..]
        "},
    );
}

#[tokio::test]
async fn test_invoke_fail() {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("invoke_invalid.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user2",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: An error occurred in the called contract [..]
        "},
    );
}

#[tokio::test]
async fn test_deploy_success_invoke_fails() {
    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("deploy_succ_invoke_fail.toml");
    let path = path.to_str().expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user3",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        path,
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall run
        error: An error occurred in the called contract [..]
        "},
    );
}
