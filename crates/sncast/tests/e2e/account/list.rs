use anyhow::Context;
use indoc::formatdoc;
use serde_json::{Value, json};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use std::{fs::File, io::Write};
use tempfile::{TempDir, tempdir};

use crate::helpers::constants::{LEDGER_PUBLIC_KEY, TEST_LEDGER_PATH_STORED};
use crate::{e2e::account::helpers::create_tempdir_with_accounts_file, helpers::runner::runner};

fn create_tempdir_with_signer_display_cases(file_name: &str) -> TempDir {
    let dir = tempdir().expect("Unable to create a temporary directory");
    let location = dir.path().join(file_name);
    let mut file = File::create(location).expect("Unable to create a temporary accounts file");

    let data = formatdoc! {r#"
        {{
            "alpha-sepolia": {{
                "ledger_user": {{
                    "public_key": "{LEDGER_PUBLIC_KEY}",
                    "address": "0x123",
                    "type": "open_zeppelin",
                    "ledger_path": "{TEST_LEDGER_PATH_STORED}"
                }},
                "ambiguous_multiple": {{
                    "public_key": "0x1",
                    "address": "0x2",
                    "private_key": "0x3",
                    "ledger_path": "{TEST_LEDGER_PATH_STORED}"
                }},
                "ambiguous_missing": {{
                    "public_key": "0x4",
                    "address": "0x5"
                }}
            }}
        }}
    "#};

    file.write_all(data.as_bytes())
        .expect("Unable to write test data to a temporary file");
    file.flush().expect("Unable to flush a temporary file");
    dir
}

#[test]
fn test_happy_case() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);

    let args = vec!["--accounts-file", &accounts_file_name, "account", "list"];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let expected = formatdoc!(
        "
        Available accounts (at {}):
        - user0:
          network: alpha-sepolia
          public key: 0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39
          signer type: private_key
          address: 0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a
          type: OpenZeppelin

        - user1:
          network: alpha-sepolia
          public key: 0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd
          signer type: private_key
          address: 0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e
          salt: 0xe2b200bbdf76c31b
          type: Ready
          class hash: 0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f

        - user2:
          network: alpha-sepolia
          public key: 0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd
          signer type: private_key
          address: 0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e
          salt: 0xe2b200bbdf76c31b
          type: Ready
          class hash: 0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f

        - user3:
          network: custom-network
          public key: 0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9
          signer type: private_key
          address: 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a

        - user4:
          network: custom-network
          public key: 0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182
          signer type: private_key
          address: 0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91
          salt: 0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064
          deployed: true

        To show private keys too, run with --display-private-keys or -p
        ",
        accounts_file_path.to_str().unwrap()
    );

    assert_stdout_contains(output, expected);
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
        &accounts_file_name,
        "account",
        "list",
        "--display-private-keys",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let expected = formatdoc!(
        "
        Available accounts (at {}):
        - user0:
          network: alpha-sepolia
          public key: 0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39
          signer type: private_key
          private key: 0x1e9038bdc68ce1d27d54205256988e85
          address: 0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a
          type: OpenZeppelin

        - user1:
          network: alpha-sepolia
          public key: 0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd
          signer type: private_key
          private key: 0x1c3495fce931c0b3ed244f55c54226441a8254deafbc7fab2e46926b4d2fdae
          address: 0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e
          salt: 0xe2b200bbdf76c31b
          type: Ready

        - user2:
          network: alpha-sepolia
          public key: 0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd
          signer type: private_key
          private key: 0x1c3495fce931c0b3ed244f55c54226441a8254deafbc7fab2e46926b4d2fdae
          address: 0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e
          salt: 0xe2b200bbdf76c31b
          type: Ready

        - user3:
          network: custom-network
          public key: 0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9
          signer type: private_key
          private key: 0xe3e70682c2094cac629f6fbed82c07cd
          address: 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a

        - user4:
          network: custom-network
          public key: 0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182
          signer type: private_key
          private key: 0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f
          address: 0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91
          salt: 0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064
          deployed: true
        ",
        accounts_file_path.to_str().unwrap()
    );

    assert_stdout_contains(output, expected);
}

#[test]
fn test_happy_case_json() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    let args = vec![
        "--json",
        "--accounts-file",
        &accounts_file_name,
        "account",
        "list",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let output_plain = output.as_stdout().to_string();
    let output_parsed: Value = serde_json::from_str(&output_plain)
        .context("Failed to parse command's output to JSON")
        .unwrap();

    let expected = json!(
        {
            "command": "account list",
            "type": "response",
            "user3": {
              "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a",
              "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
              "network": "custom-network",
              "signer_type": "private_key"
            },
            "user4": {
              "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
              "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
              "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
              "deployed": true,
              "network": "custom-network",
              "signer_type": "private_key"
            },
            "user0": {
              "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
              "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a",
              "type": "open_zeppelin",
              "network": "alpha-sepolia",
              "signer_type": "private_key"
            },
            "user1": {
              "address": "0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e",
              "class_hash": "0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f",
              "public_key": "0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd",
              "salt": "0xe2b200bbdf76c31b",
              "type": "ready",
              "network": "alpha-sepolia",
              "signer_type": "private_key"
            },
            "user2": {
              "address": "0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e",
              "class_hash": "0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f",
              "public_key": "0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd",
              "salt": "0xe2b200bbdf76c31b",
              "type": "ready",
              "network": "alpha-sepolia",
              "signer_type": "private_key"
            },
        }
    );

    assert_eq!(output_parsed, expected);
}

#[test]
fn test_happy_case_with_private_keys_json() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    let args = vec![
        "--json",
        "--accounts-file",
        &accounts_file_name,
        "account",
        "list",
        "--display-private-keys",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let output_plain = output.as_stdout().to_string();
    let output_parsed: Value = serde_json::from_str(&output_plain)
        .context("Failed to parse command's output to JSON")
        .unwrap();

    let expected = json!(
        {
          "command": "account list",
          "type": "response",
          "user3": {
              "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a",
              "private_key": "0xe3e70682c2094cac629f6fbed82c07cd",
              "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
              "network": "custom-network",
              "signer_type": "private_key"
          },
          "user4": {
            "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
            "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
            "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
            "private_key": "0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f",
            "deployed": true,
            "network": "custom-network",
            "signer_type": "private_key"
          },
          "user0": {
            "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
            "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a",
            "type": "open_zeppelin",
            "network": "alpha-sepolia",
            "private_key": "0x1e9038bdc68ce1d27d54205256988e85",
            "signer_type": "private_key"
          },
          "user1": {
            "address": "0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e",
            "class_hash": "0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f",
            "private_key": "0x1c3495fce931c0b3ed244f55c54226441a8254deafbc7fab2e46926b4d2fdae",
            "public_key": "0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd",
            "salt": "0xe2b200bbdf76c31b",
            "type": "ready",
            "network": "alpha-sepolia",
            "signer_type": "private_key"
          },
          "user2": {
            "address": "0x9613a934141dd6625748a7e066a380b3f9787f079f35ecc2f3ba934d507d4e",
            "class_hash": "0x36078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f",
            "private_key": "0x1c3495fce931c0b3ed244f55c54226441a8254deafbc7fab2e46926b4d2fdae",
            "public_key": "0x63b3a3ac141e4c007b167b27450f110c729cc0d0238541ca705b0de5144edbd",
            "salt": "0xe2b200bbdf76c31b",
            "type": "ready",
            "network": "alpha-sepolia",
            "signer_type": "private_key"
          },
        }
    );

    assert_eq!(output_parsed, expected);
}

#[test]
fn test_ledger_and_ambiguous_signers() {
    let accounts_file_name = "signer_display_accounts.json";
    let temp_dir = create_tempdir_with_signer_display_cases(accounts_file_name);

    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);

    let args = vec!["--accounts-file", accounts_file_name, "account", "list"];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let expected = formatdoc!(
        "
        Available accounts (at {}):
        - ambiguous_multiple:
          network: alpha-sepolia
          public key: 0x1
          signer type: ambiguous (only one of `private_key`, `ledger_path` may be specified)
          address: 0x2

        - ambiguous_missing:
          network: alpha-sepolia
          public key: 0x4
          signer type: ambiguous (only one of `private_key`, `ledger_path` may be specified)
          address: 0x5

        - ledger_user:
          network: alpha-sepolia
          public key: {LEDGER_PUBLIC_KEY}
          signer type: ledger
          ledger path: {TEST_LEDGER_PATH_STORED}
          address: 0x123
          type: OpenZeppelin

        To show private keys too, run with --display-private-keys or -p
        ",
        accounts_file_path.to_str().unwrap()
    );

    assert_stdout_contains(output, expected);
}

#[test]
fn test_ledger_and_ambiguous_signers_json() {
    let accounts_file_name = "signer_display_accounts.json";
    let temp_dir = create_tempdir_with_signer_display_cases(accounts_file_name);

    let args = vec![
        "--json",
        "--accounts-file",
        accounts_file_name,
        "account",
        "list",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let output_plain = output.as_stdout().to_string();
    let output_parsed: Value = serde_json::from_str(&output_plain)
        .context("Failed to parse command's output to JSON")
        .unwrap();

    let expected = json!({
        "command": "account list",
        "type": "response",
        "ledger_user": {
            "public_key": LEDGER_PUBLIC_KEY,
            "address": "0x123",
            "type": "open_zeppelin",
            "network": "alpha-sepolia",
            "signer_type": "ledger",
            "ledger_path": TEST_LEDGER_PATH_STORED,
        },
        "ambiguous_multiple": {
            "public_key": "0x1",
            "address": "0x2",
            "network": "alpha-sepolia",
            "signer_type": "ambiguous",
        },
        "ambiguous_missing": {
            "public_key": "0x4",
            "address": "0x5",
            "network": "alpha-sepolia",
            "signer_type": "ambiguous",
        },
    });

    assert_eq!(output_parsed, expected);
}

#[test]
fn test_accounts_file_does_not_exist() {
    let accounts_file_name = "some_inexistent_file.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec!["--accounts-file", &accounts_file_name, "account", "list"];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert!(output.as_stdout().is_empty());

    let expected = "Error: Accounts file = some_inexistent_file.json does not exist! \
        If you do not have an account create one with `account create` command \
        or if you're using a custom accounts file, make sure \
        to supply correct path to it with `--accounts-file` argument.";

    assert_stderr_contains(output, expected);
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

    let args = vec!["--accounts-file", &accounts_file_name, "account", "list"];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());
    assert_stdout_contains(
        output,
        format!(
            "No accounts available at {}",
            accounts_file_path.to_str().unwrap()
        ),
    );
}
