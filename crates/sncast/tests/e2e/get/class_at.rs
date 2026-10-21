use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use serde::Deserialize;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use sncast::helpers::constants::MAP_CONTRACT_ADDRESS_SEPOLIA;
use starknet_rust::core::types::ContractClass;

use test_case::test_case;

use super::class::ContractVariant;

#[test_case(ContractVariant::Sierra)]
#[test_case(ContractVariant::Legacy)]
#[tokio::test]
async fn test_pretty_output(variant: ContractVariant) {
    let args = vec!["get", "class-at", variant.class_address(), "--url", URL];

    let output = runner(&args).assert().success();

    let res: ContractClass = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass");

    let output = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    match (variant, res) {
        (ContractVariant::Sierra, ContractClass::Sierra(_))
        | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
        _ => panic!("Expected {variant:?}"),
    }

    insta::assert_snapshot!(format!("get-class-at-pretty-{variant:?}"), output);
}

#[test_case(ContractVariant::Sierra)]
#[test_case(ContractVariant::Legacy)]
#[tokio::test]
async fn test_json_output(variant: ContractVariant) {
    #[derive(Deserialize)]
    struct Response {
        #[serde(flatten)]
        contract: ContractClass,
        command: String,
        #[serde(rename = "type")]
        res_type: String,
    }

    let args = vec![
        "--json",
        "get",
        "class-at",
        variant.class_address(),
        "--url",
        URL,
    ];

    let output = runner(&args).assert().success();

    let res: Response = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass with additional data");

    assert_eq!(&res.res_type, "response");
    assert_eq!(&res.command, "get class-at");

    match (variant, res.contract) {
        (ContractVariant::Sierra, ContractClass::Sierra(_))
        | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
        _ => panic!("Expected {variant:?}"),
    }
}

#[tokio::test]
async fn test_non_existent_address() {
    let args = vec!["get", "class-at", "0x43", "--url", URL];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get class-at
        Error: There is no contract at the specified address
        "},
    );
}

#[tokio::test]
async fn test_non_existent_address_json() {
    let args = vec!["--json", "get", "class-at", "0x43", "--url", URL];

    let output = runner(&args).assert().failure();

    let res: serde_json::Value = serde_json::from_str(output.as_stderr()).unwrap();
    let expected = serde_json::json!({
        "command": "get class-at",
        "type": "error",
        "error": "There is no contract at the specified address"
    });

    assert_eq!(res, expected);
}

#[tokio::test]
async fn test_with_block_id() {
    let args = vec![
        "get",
        "class-at",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--block-id",
        "latest",
        "--url",
        URL,
    ];

    let output = runner(&args).assert().success();

    let _: ContractClass = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass");
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec![
        "get",
        "class-at",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--block-id",
        "invalid_block",
        "--url",
        URL,
    ];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get class-at
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}

#[tokio::test]
async fn test_with_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec!["get", "class-at", "@map", "--url", URL];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    let _: ContractClass = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass");
}

#[tokio::test]
async fn test_with_unknown_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec!["get", "class-at", "@unknown", "--url", URL];

    let output = runner(&args).current_dir(tempdir.path()).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: get class-at
            Error: Invalid contract address

            Caused by:
                Alias `unknown` not found in config
        "},
    );
}
