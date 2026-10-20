use serde::Deserialize;
use starknet_rust::core::types::ContractClass;

use crate::helpers::constants::{
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::runner::runner;

use test_case::test_case;

#[derive(Copy, Clone, Debug)]
enum ContractVariant {
    Sierra,
    Legacy,
}

impl ContractVariant {
    fn class_hash(self) -> &'static str {
        match self {
            ContractVariant::Sierra => MAP_CONTRACT_CLASS_HASH_SEPOLIA,
            ContractVariant::Legacy => DEVNET_OZ_CLASS_HASH_CAIRO_0,
        }
    }
}

#[test_case(ContractVariant::Sierra)]
#[test_case(ContractVariant::Legacy)]
#[tokio::test]
async fn test_pretty_output(variant: ContractVariant) {
    let args = vec!["get", "class", variant.class_hash(), "--url", URL];

    let output = runner(&args).assert().success();

    let res: ContractClass = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass");

    let output = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    assert!(
        output.lines().count() > 1,
        "Expected result to be pretty-printed"
    );

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

    let args = vec!["--json", "get", "class", variant.class_hash(), "--url", URL];

    let output = runner(&args).assert().success();

    let res: Response = serde_json::from_slice(&output.get_output().stdout)
        .expect("Failed to parse as ContractClass with additional data");

    assert_eq!(&res.res_type, "response");
    assert_eq!(&res.command, "get class");

    match (variant, res.contract) {
        (ContractVariant::Sierra, ContractClass::Sierra(_))
        | (ContractVariant::Legacy, ContractClass::Legacy(_)) => {}
        _ => panic!("Expected {variant:?}"),
    }
}
