use crate::helpers::{
    constants::CONTRACTS_DIR,
    fixtures::{copy_directory_to_tempdir, duplicate_contract_directory_with_salt},
    runner::runner,
};
use indoc::indoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use std::process::Stdio;
use tempfile::tempdir;

#[test]
fn test_happy_case_get_class_hash() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "human_readable",
    );

    let args = vec!["utils", "class-hash", "--contract-name", "Map"];

    let snapbox = runner(&args).current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(output, indoc! {r"Class Hash: 0x0[..]"});
}

#[test]
fn test_happy_case_get_class_hash_from_sierra_file() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "class_hash_from_sierra_file",
    );

    let build_output = ScarbCommand::new()
        .arg("build")
        .current_dir(contract_path.path())
        .command()
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb build`");

    assert!(build_output.status.success(), "`scarb build` failed");

    let sierra_path = contract_path
        .path()
        .join("target/dev/map_Map.contract_class.json");
    assert!(
        sierra_path.exists(),
        "sierra artifact not found at {sierra_path:?}"
    );
    let sierra_path = sierra_path.to_str().unwrap();

    let args = vec!["utils", "class-hash", "--sierra-file", sierra_path];

    let snapbox = runner(&args).current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(output, indoc! {r"Class Hash: 0x0[..]"});
}

#[test]
fn test_class_hash_requires_contract_name_or_sierra_file() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec!["utils", "class-hash"];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          <--contract-name <CONTRACT>|--sierra-file <SIERRA_FILE>>
        "},
    );
}

#[test]
fn test_class_hash_conflicting_contract_source() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec![
        "utils",
        "class-hash",
        "--contract-name",
        "Map",
        "--sierra-file",
        "path/to/sierra.json",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "error: the argument '--contract-name <CONTRACT>' cannot be used with '--sierra-file <SIERRA_FILE>'",
    );
}

#[test]
fn test_class_hash_sierra_file_does_not_exist() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec![
        "utils",
        "class-hash",
        "--sierra-file",
        "/nonexistent/path/contract.contract_class.json",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: utils class-hash
        Error: Failed to read sierra file at [..]contract_class.json
        "},
    );
}

#[test]
fn test_class_hash_sierra_file_invalid_json() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let invalid_sierra_path = temp_dir.path().join("invalid_contract_class.json");
    fs::write(&invalid_sierra_path, r#"{"not": "a valid sierra"}"#).unwrap();
    let invalid_sierra_path = invalid_sierra_path.to_str().unwrap();

    let args = vec!["utils", "class-hash", "--sierra-file", invalid_sierra_path];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: utils class-hash
        Error: Failed to parse sierra file
        "},
    );
}

#[test]
fn test_errors_on_ambiguous_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let args = vec!["utils", "class-hash", "--contract-name", "HelloStarknet"];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r#"
        Error: Found more than one contract named "HelloStarknet" at: duplicate_contract_name::first_contract::HelloStarknet, duplicate_contract_name::second_contract::HelloStarknet
        "#},
    );
}
