use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use serde::Deserialize;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
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

    insta::assert_snapshot!(output);

    match (variant, res) {
        (ContractVariant::Sierra, ContractClass::Sierra(_))
        | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
        _ => panic!("Expected {variant:?}"),
    }
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
