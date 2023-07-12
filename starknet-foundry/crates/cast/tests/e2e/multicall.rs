use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use std::path::Path;

static USERNAME: &str = "user2";
static USERNAME2: &str = "user3";

#[tokio::test]
async fn test_happy_case() {
    let args = default_cli_args(USERNAME.to_string());
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path).join("crates/cast/tests/data/multicall_configs/deploy_invoke.toml");
    let path_str = path.to_str().expect("failed converting path to str");

    args.append(&mut vec!["multicall", "--path", path_str]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(out.stderr.is_empty());
    assert!(stdout_str.contains("command: Deploy"));
    assert!(stdout_str.contains("command: Invoke"));
}

#[tokio::test]
async fn test_invalid_path() {
    let args = default_cli_args(USERNAME.to_string());
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    args.append(&mut vec!["multicall", "--path", "non-existent"]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    assert!(out.stdout.is_empty());
    let stderr_str =
        std::str::from_utf8(&out.stderr[..]).expect("failed to convert stderr to string");
    assert!(stderr_str.contains("No such file or directory"));
}

#[tokio::test]
async fn test_deploy_fail() {
    let args = default_cli_args(USERNAME.to_string());
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path =
        Path::new(&path).join("crates/cast/tests/data/multicall_configs/deploy_invalid.toml");
    let path_str = path.to_str().expect("failed converting path to str");

    args.append(&mut vec!["multicall", "--path", path_str]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(stderr_str.contains("Class with hash 0x1 is not declared"));
}

#[tokio::test]
async fn test_invoke_fail() {
    let args = default_cli_args(USERNAME.to_string());
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path =
        Path::new(&path).join("crates/cast/tests/data/multicall_configs/invoke_invalid.toml");
    let path_str = path.to_str().expect("failed converting path to str");

    args.append(&mut vec!["multicall", "--path", path_str]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(out.stdout.is_empty());
    assert!(stderr_str.contains("There is no contract at the specified address"));
}

#[tokio::test]
async fn test_deploy_success_invoke_fails() {
    let args = default_cli_args(USERNAME2.to_string());
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join("crates/cast/tests/data/multicall_configs/deploy_succ_invoke_fail.toml");
    let path_str = path.to_str().expect("failed converting path to str");

    args.append(&mut vec!["multicall", "--path", path_str]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: Deploy"));
    assert!(stderr_str.contains("error: There is no contract at the specified address"));
}
