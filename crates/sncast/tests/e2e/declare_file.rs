use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    create_and_deploy_oz_account, duplicate_contract_directory_with_salt, get_accounts_path,
    join_tempdirs,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use std::process::Stdio;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "declare_file_happy",
    );

    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let build_output = ScarbCommand::new()
        .arg("build")
        .current_dir(tempdir.path())
        .command()
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb build`");

    assert!(build_output.status.success(), "`scarb build` failed");
    let sierra_path = tempdir
        .path()
        .join("target/dev/map_Map.contract_class.json");
    assert!(
        sierra_path.exists(),
        "sierra artifact not found at {sierra_path:?}"
    );
    let sierra_path = sierra_path.to_str().unwrap();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-file",
        "--sierra-path",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x[..]
        Transaction Hash: 0x[..]

        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    "},
    );
}

#[tokio::test]
async fn test_file_does_not_exist() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-file",
        "--sierra-path",
        "/nonexistent/path/contract.contract_class.json",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-file
        Error: Failed to read Sierra file at [..]contract_class.json: No such file or directory [..]
        "},
    );
}

#[tokio::test]
async fn test_invalid_sierra_json() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let invalid_sierra_path = temp_dir.path().join("invalid_contract_class.json");
    fs::write(&invalid_sierra_path, r#"{"not": "a valid sierra"}"#).unwrap();
    let invalid_sierra_path = invalid_sierra_path.to_str().unwrap().to_string();

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-file",
        "--sierra-path",
        invalid_sierra_path.as_str(),
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-file
        Error: Failed to parse Sierra file as contract class: missing field `sierra_program` at line 1 column 25
        "},
    );
}

#[tokio::test]
async fn test_contract_already_declared() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "declare_file_already_declared",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let build_output = ScarbCommand::new()
        .arg("build")
        .current_dir(tempdir.path())
        .command()
        .output()
        .expect("Failed to run `scarb build`");
    assert!(build_output.status.success(), "`scarb build` failed");

    let sierra_path = tempdir
        .path()
        .join("target/dev/map_Map.contract_class.json");
    let sierra_path = sierra_path.to_str().unwrap();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-file",
        "--sierra-path",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    runner(&args).current_dir(tempdir.path()).assert().success();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-file",
        "--sierra-path",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-file
        Error: Contract with class hash 0x0[..] is already declared
        "},
    );
}

#[test]
fn test_no_sierra_path_specified() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let args = vec!["declare-file", "--url", URL];
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          --sierra-path <SIERRA_PATH>

        Usage: sncast declare-file --sierra-path <SIERRA_PATH> --url <URL>

        For more information, try '--help'.
        "},
    );
}
