use crate::helpers::constants::{ACCOUNT_FILE_PATH, MULTICALL_CONFIGS_DIR, URL};
use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use std::path::Path;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let temp_dir = tempdir().expect("Unable to create temporary directory");

    let config_path = "./deploy_invoke.toml";
    let account_path = "./accounts.json";

    let root_path = project_root::get_project_root().expect("failed to get project root path");

    fs_extra::file::copy(
        Path::new(&root_path)
            .join(MULTICALL_CONFIGS_DIR)
            .join(config_path),
        temp_dir.path().join(config_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy config file");

    fs_extra::file::copy(
        ACCOUNT_FILE_PATH,
        temp_dir.path().join(account_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts file");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        account_path,
        "--account",
        "user2",
        "multicall",
        "run",
        "--path",
        config_path,
    ];
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command stderr to string");
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    assert!(stdout_str.contains("command: multicall"));
}

#[tokio::test]
async fn test_calldata_ids() {
    let temp_dir = tempdir().expect("Unable to create temporary directory");

    let config_path = "./deploy_invoke_calldata_ids.toml";
    let account_path = "./accounts.json";

    let root_path = project_root::get_project_root().expect("failed to get project root path");

    fs_extra::file::copy(
        Path::new(&root_path)
            .join(MULTICALL_CONFIGS_DIR)
            .join(config_path),
        temp_dir.path().join(config_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy config file");

    fs_extra::file::copy(
        ACCOUNT_FILE_PATH,
        temp_dir.path().join(account_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts file");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        account_path,
        "--account",
        "user5",
        "multicall",
        "run",
        "--path",
        config_path,
    ];
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command stderr to string");
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );
    assert!(stdout_str.contains("command: multicall"));
}

#[tokio::test]
async fn test_invalid_path() {
    let mut args = default_cli_args();
    args.append(&mut vec!["--account", "user2"]);

    args.append(&mut vec!["multicall", "run", "--path", "non-existent"]);

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
    let temp_dir = tempdir().expect("Unable to create temporary directory");

    let config_path = "./deploy_invalid.toml";
    let account_path = "./accounts.json";

    let root_path = project_root::get_project_root().expect("failed to get project root path");

    fs_extra::file::copy(
        Path::new(&root_path)
            .join(MULTICALL_CONFIGS_DIR)
            .join(config_path),
        temp_dir.path().join(config_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy config file");

    fs_extra::file::copy(
        ACCOUNT_FILE_PATH,
        temp_dir.path().join(account_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts file");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        account_path,
        "--account",
        "user2",
        "multicall",
        "run",
        "--path",
        config_path,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(stderr_str.contains("An error occurred in the called contract"));
}

#[tokio::test]
async fn test_invoke_fail() {
    let temp_dir = tempdir().expect("Unable to create temporary directory");

    let config_path = "./invoke_invalid.toml";
    let account_path = "./accounts.json";

    let root_path = project_root::get_project_root().expect("failed to get project root path");

    fs_extra::file::copy(
        Path::new(&root_path)
            .join(MULTICALL_CONFIGS_DIR)
            .join(config_path),
        temp_dir.path().join(config_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy config file");
    fs_extra::file::copy(
        ACCOUNT_FILE_PATH,
        temp_dir.path().join(account_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts file");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        account_path,
        "--account",
        "user2",
        "multicall",
        "run",
        "--path",
        config_path,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(out.stdout.is_empty());
    assert!(stderr_str.contains("An error occurred in the called contract"));
}

#[tokio::test]
async fn test_deploy_success_invoke_fails() {
    let temp_dir = tempdir().expect("Unable to create temporary directory");

    let config_path = "./deploy_succ_invoke_fail.toml";
    let account_path = "./accounts.json";

    let root_path = project_root::get_project_root().expect("failed to get project root path");

    fs_extra::file::copy(
        Path::new(&root_path)
            .join(MULTICALL_CONFIGS_DIR)
            .join(config_path),
        temp_dir.path().join(config_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy config file");
    fs_extra::file::copy(
        ACCOUNT_FILE_PATH,
        temp_dir.path().join(account_path),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts file");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        account_path,
        "--account",
        "user3",
        "multicall",
        "run",
        "--path",
        config_path,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = String::from_utf8(snapbox.assert().success().get_output().stderr.clone()).unwrap();

    assert!(output.contains("An error occurred in the called contract"));
}
