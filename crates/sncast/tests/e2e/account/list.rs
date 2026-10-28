use serde_json::Value;
use shared::test_utils::output_assert::AsOutput;
use std::fs;
use tempfile::tempdir;

use crate::{e2e::account::helpers::create_tempdir_with_accounts_file, helpers::runner::runner};

#[test]
fn test_happy_case() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);
    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);
    let args = vec!["--accounts-file", accounts_file_name, "account", "list"];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    insta::assert_snapshot!(
        output
            .as_stdout()
            .replace(accounts_file_path.to_str().unwrap(), "[ACCOUNTS_FILE]")
    );
}

#[test]
fn test_happy_case_with_private_keys() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);
    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);
    let args = vec![
        "--accounts-file",
        accounts_file_name,
        "account",
        "list",
        "--display-private-keys",
    ];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    insta::assert_snapshot!(
        output
            .as_stdout()
            .replace(accounts_file_path.to_str().unwrap(), "[ACCOUNTS_FILE]")
    );
}

#[test]
fn test_happy_case_json() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);
    let args = vec![
        "--json",
        "--accounts-file",
        accounts_file_name,
        "account",
        "list",
    ];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    let output: Value = serde_json::from_str(output.as_stdout()).unwrap();
    insta::assert_json_snapshot!(output);
}

#[test]
fn test_happy_case_with_private_keys_json() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);
    let args = vec![
        "--json",
        "--accounts-file",
        accounts_file_name,
        "account",
        "list",
        "--display-private-keys",
    ];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    let output: Value = serde_json::from_str(output.as_stdout()).unwrap();
    insta::assert_json_snapshot!(output);
}

#[test]
fn test_accounts_with_same_name_in_different_networks_are_listed() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    fs::write(
        temp_dir.path().join(accounts_file_name),
        r#"{
            "alpha-sepolia": {
                "shared": { "private_key": "0x1", "public_key": "0x1" }
            },
            "custom-network": {
                "shared": { "private_key": "0x2", "public_key": "0x2" }
            }
        }"#,
    )
    .expect("Unable to write accounts file");
    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);

    let args = vec!["--accounts-file", accounts_file_name, "account", "list"];
    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    insta::assert_snapshot!(
        "same-name-text",
        output
            .as_stdout()
            .replace(accounts_file_path.to_str().unwrap(), "[ACCOUNTS_FILE]")
    );

    let args = vec![
        "--json",
        "--accounts-file",
        accounts_file_name,
        "account",
        "list",
    ];
    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    let output: Value = serde_json::from_str(output.as_stdout()).unwrap();
    insta::assert_json_snapshot!("same-name-json", output);
}

#[test]
fn test_accounts_file_does_not_exist() {
    let accounts_file_name = "some_inexistent_file.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let args = vec!["--accounts-file", accounts_file_name, "account", "list"];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .failure();

    assert!(output.as_stdout().is_empty());
    insta::assert_snapshot!(output.as_stderr());
}

#[test]
fn test_no_accounts_available() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, false);
    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);
    let args = vec!["--accounts-file", accounts_file_name, "account", "list"];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(output.as_stderr().is_empty());
    insta::assert_snapshot!(
        output
            .as_stdout()
            .replace(accounts_file_path.to_str().unwrap(), "[ACCOUNTS_FILE]")
    );
}
