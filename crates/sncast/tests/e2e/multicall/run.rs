use crate::helpers::constants::{ACCOUNT_FILE_PATH, MULTICALL_CONFIGS_DIR, URL};
use crate::helpers::fixtures::{create_and_deploy_oz_account, join_tempdirs};
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use std::path::Path;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("ready"; "ready_account")]
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

    let snapbox = runner(&args).env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1");
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_eq(indoc! {r"
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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_eq(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_dry_run() {
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
        "--dry-run",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Dry run completed

            Overall Fee: [..] Fri (~[..] STRK)
            "
        },
    );
}

#[tokio::test]
async fn test_dry_run_detailed() {
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
        "--dry-run",
        "--detailed",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Dry run completed

            Overall Fee: [..] Fri (~[..] STRK)
            L1 Gas Consumed:      [..]
            L1 Gas Price:         [..]
            L2 Gas Consumed:      [..]
            L2 Gas Price:         [..]
            L1 Data Gas Consumed: [..]
            L1 Data Gas Price:    [..]
            "
        },
    );
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
    let output = snapbox.assert().failure();

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
    let output = snapbox.assert().failure();

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
    let output = snapbox.assert().failure();

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

    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall run
        Error: Transaction execution error [..]
        "},
    );
}

#[tokio::test]
async fn test_run_id_overrides_alias() {
    let account_dir = create_and_deploy_oz_account().await;
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&account_dir, &config_dir);

    let path = project_root::get_project_root().expect("failed to get project root path");
    // IDs defined in multicall file take precedence over the aliases defined in `snfoundry.toml`
    // Otherwise this test would fail as `shadowed` defined in config aliases points to non-existent address
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("multicall_id_overrides_alias.toml");
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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(config_dir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_eq(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_run_alias_from_config_only() {
    let account_dir = create_and_deploy_oz_account().await;
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&account_dir, &config_dir);

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("multicall_with_alias_from_config.toml");
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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(config_dir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_eq(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}

#[tokio::test]
async fn test_run_with_unknown_alias_in_inputs() {
    let account_dir = create_and_deploy_oz_account().await;
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&account_dir, &config_dir);

    let path = project_root::get_project_root().expect("failed to get project root path");
    let path = Path::new(&path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("multicall_with_unknown_alias_in_inputs.toml");
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

    let output = runner(&args)
        .current_dir(config_dir.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: multicall run
            Error: `@unknown`: not found as multicall step id or in [sncast.<profile>.aliases]
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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert();

    let stderr_str = output.as_stderr();
    assert!(
        stderr_str.is_empty(),
        "Multicall error, stderr: \n{stderr_str}",
    );

    output.stdout_eq(indoc! {r"
        Success: Multicall completed

        Transaction Hash: 0x[..]

        To see invocation details, visit:
        transaction: [..]
    "});
}
