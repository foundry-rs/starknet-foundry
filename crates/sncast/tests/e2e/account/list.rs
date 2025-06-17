use anyhow::Context;
use indoc::formatdoc;
use serde_json::{Value, json};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
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
          address: 0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a
          type: OpenZeppelin

        - user3:
          network: custom-network
          public key: 0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9
          address: 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a

        - user4:
          network: custom-network
          public key: 0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182
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
          private key: 0x1e9038bdc68ce1d27d54205256988e85
          public key: 0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39
          address: 0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a
          type: OpenZeppelin

        - user3:
          network: custom-network
          private key: 0xe3e70682c2094cac629f6fbed82c07cd
          public key: 0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9
          address: 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a

        - user4:
          network: custom-network
          private key: 0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f
          public key: 0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182
          address: 0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91
          salt: 0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064
          deployed: true
        ",
        accounts_file_path.to_str().unwrap()
    );

    assert_stdout_contains(output, expected);
}

#[test]
fn test_happy_case_int_format() {
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
          private key: 40625681471685359029804301037638028933
          public key: 1344783310009133679377326611173868415467858937993314384123595886829324020537
          address: 2243801221490456549135145738506528093449479171219304558490820973710020585018
          type: OpenZeppelin

        - user3:
          network: custom-network
          private key: 302934307671667531413257853548643485645
          public key: 3571077580641057962019375980836964323430604474979724507958294224671833227961
          address: 3562055384976875123115280411327378123839557441680670463096306030682092229914

        - user4:
          network: custom-network
          private key: 3278793552591849920356004222758625564696225216399892679169751024513874444319
          public key: 1912535824053513524044241194146845716933313499165320136252999660831350960514
          address: 3528166482527127075479645747648835917396168866434791003742065878852209458833
          salt: 2393464100970799969082151102468006585314800480204341526354458084672178651236
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
            "user3": {
              "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a",
              "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
              "network": "custom-network"
            },
            "user4": {
              "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
              "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
              "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
              "deployed": true,
              "network": "custom-network"
            },
            "user0": {
              "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
              "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a",
              "type": "open_zeppelin",
              "network": "alpha-sepolia"
            }
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
          "user3": {
              "address": "0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a",
              "private_key": "0xe3e70682c2094cac629f6fbed82c07cd",
              "public_key": "0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9",
              "network": "custom-network"
          },
          "user4": {
            "public_key": "0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182",
            "address": "0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91",
            "salt": "0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064",
            "private_key": "0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f",
            "deployed": true,
            "network": "custom-network"
          },
          "user0": {
            "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
            "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a",
            "type": "open_zeppelin",
            "network": "alpha-sepolia",
            "private_key": "0x1e9038bdc68ce1d27d54205256988e85",
          }
        }
    );

    assert_eq!(output_parsed, expected);
}

#[test]
fn test_happy_case_with_private_keys_json_int_format() {
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
          "user3": {
            "address": "3562055384976875123115280411327378123839557441680670463096306030682092229914",
            "private_key": "302934307671667531413257853548643485645",
            "public_key": "3571077580641057962019375980836964323430604474979724507958294224671833227961",
            "network": "custom-network"
          },
          "user4": {
            "public_key": "1912535824053513524044241194146845716933313499165320136252999660831350960514",
            "address": "3528166482527127075479645747648835917396168866434791003742065878852209458833",
            "salt": "2393464100970799969082151102468006585314800480204341526354458084672178651236",
            "private_key": "3278793552591849920356004222758625564696225216399892679169751024513874444319",
            "deployed": true,
            "network": "custom-network"
            },
          "user0": {
            "public_key": "1344783310009133679377326611173868415467858937993314384123595886829324020537",
            "private_key": "40625681471685359029804301037638028933",
            "address": "2243801221490456549135145738506528093449479171219304558490820973710020585018",
            "type": "open_zeppelin",
            "network": "alpha-sepolia"
          }
        }
    );

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
