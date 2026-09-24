use indoc::indoc;

use crate::helpers::{constants::URL, runner::runner};

#[tokio::test]
async fn test_pretty_output() {
    let args = vec!["get", "chain-id", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        Success: Chain ID retrieved

        Chain ID:   0x534e5f5345504f4c4941
        Chain Name: SN_SEPOLIA
    "#});
}

#[tokio::test]
async fn test_json_output() {
    let args = vec!["get", "chain-id", "--url", URL, "--json"];
    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(json["command"], "get chain-id");
    assert_eq!(json["type"], "response");
    assert_eq!(json["chain_id"], "0x534e5f5345504f4c4941");
    assert_eq!(json["chain_name"], "SN_SEPOLIA");
}
