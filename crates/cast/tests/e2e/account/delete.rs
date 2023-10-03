use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use std::fs::File;
use std::io::Write;

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

    snapbox.assert().stderr_matches(indoc! {r#"
    command: account delete
    error: No accounts defined for network goerli0-network
    "#});
}

#[tokio::test]
pub async fn test_account_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec!["account", "delete", "--name", "user99"]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
    command: account delete
    error: Account with name user99 does not exist
    "#});
}

#[tokio::test]
pub async fn test_no_confirmation() {
    // Creating dummy accounts test file
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
            }
        }
    }
    "#};

    let mut file =
        File::create("temp_accounts3.json").expect("Could not create temporary accounts file!");
    file.write_all(json_data.as_bytes())
        .expect("Could not write temporary testing accounts");
    let _ = file.flush();

    // Now delete dummy account
    let args = vec![
        "--url",
        "http://127.0.0.1:5050/",
        "--accounts-file",
        "temp_accounts3.json",
        "account",
        "delete",
        "--name",
        "user3",
        "--network",
        "alpha-goerli2",
    ];

    // Run test with a negative user input
    let snapbox = Command::new(cargo_bin!("sncast")).args(args).stdin("n");
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let _ = std::fs::remove_file("temp_accounts3.json");
    assert!(stdout_str.contains("Delete cancelled"));
}

#[tokio::test]
pub async fn test_happy_case() {
    // Creating dummy accounts test file
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

    let mut file =
        File::create("temp_accounts1.json").expect("Could not create temporary accounts file!");
    file.write_all(json_data.as_bytes())
        .expect("Could not write temporary testing accounts");
    let _ = file.flush();

    // Now delete dummy account
    let args = vec![
        "--url",
        "http://127.0.0.1:5050/",
        "--accounts-file",
        "temp_accounts1.json",
        "account",
        "delete",
        "--name",
        "user3",
        "--network",
        "alpha-goerli2",
    ];

    // Run test with an affirmative user input
    let snapbox = Command::new(cargo_bin!("sncast")).args(args).stdin("Y");
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let _ = std::fs::remove_file("temp_accounts1.json");
    assert!(stdout_str.contains("Account successfully removed"));
}

#[tokio::test]
pub async fn test_happy_case_with_remove_profile() {
    // Creating dummy accounts and scarb.toml test file
    let mut json_data = indoc! {r#"
    {
        "alpha-goerli": {
            "__default__": {
                "private_key": "0x421b62d2cba1fd39798d719a6cee5f599afc79b0bacbea50f76215057c068dd",
                "public_key": "0x12d3ad59161fd2a72d5bc8501bb2f2ca1acd34706d2dfa31a90aadb4b41e050",
                "address": "0x20f8c63faff27a0c5fe8a25dc1635c40c971bf67b8c35c6089a998649dfdfcb"
            },
            "user0": {
                "private_key": "0x1e9038bdc68ce1d27d54205256988e85",
                "public_key": "0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39",
                "address": "0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a"
            },
            "user1": {
                "private_key": "0x55ae34c86281fbd19292c7e3bfdfceb4",
                "public_key": "0xe2d3d7080bfc665e0060a06e8e95c3db3ff78a1fec4cc81ddc87e49a12e0a",
                "salt": "0x14b6b215424909f34f417ddd7cbaca48de2d505d03c92467367d275e847d252",
                "address": "0x112b61669a84940af30544786a6b615755b192173c4665a7d38fb3aa0541cac",
                "deployed": true
            }
        }
    }
    "#};

    let mut file =
        File::create("temp_accounts2.json").expect("Could not create temporary accounts file!");
    file.write_all(json_data.as_bytes())
        .expect("Could not write temporary testing accounts");
    let _ = file.flush();

    json_data = indoc! {r#"
    [dependencies]
    starknet = "1.1.1"

    [dependencies.snforge_std]
    git = "https://github.com/foundry-rs/starknet-foundry"
    tag = "v0.6.0"

    [package]
    name = "mypackage"
    version = "0.1.0"

    [[target.starknet-contract]]
    casm = true

    [tool.sncast]
    url = "http://127.0.0.1:5050/rpc"

    [tool.sncast.user0]
    account = "a4"
    accounts-file = "./myfile.json"
    url = "http://127.0.0.1:5050/rpc"

    [tool.sncast.user1]
    account = "a4"
    accounts-file = "./myfile.json"
    url = "http://127.0.0.1:5050/rpc"
    "#};

    let mut file = File::create("Scarb.toml").expect("Could not create temporary scarb file!");
    file.write_all(json_data.as_bytes())
        .expect("Could not write temporary scarb accounts");
    let _ = file.flush();

    // Now delete dummy account from accountd file and his profile from scarb
    let args = vec![
        "--url",
        "http://127.0.0.1:5050/",
        "--accounts-file",
        "temp_accounts2.json",
        "account",
        "delete",
        "--name",
        "user1",
        "--network",
        "alpha-goerli",
        "--delete-profile",
    ];

    // Run test with an affirmative user input
    let snapbox = Command::new(cargo_bin!("sncast")).args(args).stdin("Y");
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let _ = std::fs::remove_file("temp_accounts2.json");
    let _ = std::fs::remove_file("Scarb.toml");
    assert!(stdout_str.contains("Account successfully removed"));
}
