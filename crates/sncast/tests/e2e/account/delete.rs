use crate::helpers::constants::URL;
use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use std::path::Path;
use tempfile::{tempdir};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::test]
pub async fn test_no_accounts_in_network() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "account",
        "delete",
        "--name",
        "user99",
        "--network",
        "goerli0-network",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
    command: account delete
    error: No accounts defined for network = goerli0-network
    "});
}

#[tokio::test]
pub async fn test_account_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec!["account", "delete", "--name", "user99"]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
    command: account delete
    error: Account with name user99 does not exist
    "});
}

#[tokio::test]
pub async fn test_delete_abort() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    // Creating dummy accounts test file
    let accounts_file_path = temp_dir.path().join("temp_accounts.json");
    create_dummy_accounts_file(&accounts_file_path).await;

    // Now delete dummy account
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "temp_accounts.json",
        "account",
        "delete",
        "--name",
        "user3",
        "--network",
        "alpha-goerli2",
    ];

    // Run test with a negative user input
    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args)
        .stdin("n");

    snapbox.assert().stderr_matches(indoc! {r"
    command: account delete
    error: Delete aborted
    "});
}

#[tokio::test]
pub async fn test_happy_case() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    // Creating dummy accounts test file
    let accounts_file_path = temp_dir.path().join("temp_accounts.json");
    create_dummy_accounts_file(&accounts_file_path).await;

    // Now delete dummy account
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "temp_accounts.json",
        "account",
        "delete",
        "--name",
        "user3",
        "--network",
        "alpha-goerli2",
    ];

    // Run test with an affirmative user input
    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args)
        .stdin("Y");
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(stdout_str.contains("Account successfully removed"));
}

#[tokio::test]
pub async fn test_happy_case_without_network_args() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    // Creating dummy accounts test file
    let accounts_file_path = temp_dir.path().join("temp_accounts.json");
    create_dummy_accounts_file(&accounts_file_path).await;

    // Now delete dummy account
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "temp_accounts.json",
        "account",
        "delete",
        "--name",
        "user0",
    ];

    // Run test with an affirmative user input
    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args)
        .stdin("Y");
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(stdout_str.contains("Account successfully removed"));
}

#[tokio::test]
pub async fn test_happy_case_with_yes_flag() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    // Creating dummy accounts test file
    let accounts_file_path = temp_dir.path().join("temp_accounts.json");
    create_dummy_accounts_file(&accounts_file_path).await;

    // Now delete dummy account
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "temp_accounts.json",
        "account",
        "delete",
        "--name",
        "user3",
        "--network",
        "alpha-goerli2",
        "--yes",
    ];

    // Run test with no additional user input
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(out.stderr.is_empty());
    assert!(stdout_str.contains("Account successfully removed"));
}

async fn create_dummy_accounts_file(file_path: &Path) {
    let json_data = indoc! {r#"
    {
        "alpha-goerli": {
            "user0": {
                "private_key": "0x1e9038bdc68ce1d27d54205256988e85",
                "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
                "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a"
            }
        },
        "alpha-goerli2": {
            "user3": {
                "private_key": "0xe3e70682c2094cac629f6fbed82c07cd",
                "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
                "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a"
            },
            "user4": {
                "private_key": "0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f",
                "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
                "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
                "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
                "deployed": true
            }
        }
    }
    "#};

    let mut file = File::create(file_path)
        .await
        .expect("Could not create temporary accounts file!");
    file.write_all(json_data.as_bytes())
        .await
        .expect("Could not write temporary testing accounts");
    let _ = file.flush().await;
}
