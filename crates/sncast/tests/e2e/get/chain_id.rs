use indoc::indoc;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::helpers::{constants::URL, runner::runner};

#[tokio::test]
async fn test_pretty_output() {
    let args = vec!["get", "chain-id", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {"
        Success: Chain ID retrieved

        Chain ID:   0x534e5f5345504f4c4941
        Chain Name: SN_SEPOLIA
    "});
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

#[tokio::test]
async fn test_malformed_chain_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "starknet_specVersion"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": "0.10.0"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "starknet_chainId"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": "0x610062"
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    let args = vec!["get", "chain-id", "--url", &url];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {"
        Success: Chain ID retrieved

        Chain ID: 0x610062
    "});
}

#[tokio::test]
async fn test_malformed_chain_id_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "starknet_specVersion"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": "0.10.0"
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "starknet_chainId"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": "0x610062"
        })))
        .mount(&mock_server)
        .await;

    let url = mock_server.uri();
    let args = vec!["get", "chain-id", "--url", &url, "--json"];
    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(json["command"], "get chain-id");
    assert_eq!(json["type"], "response");
    assert_eq!(json["chain_id"], "0x610062");
    assert!(json.get("chain_name").is_none());
}
