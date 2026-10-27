use crate::helpers::{constants::URL, runner::runner};
use indoc::indoc;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_pretty_output_not_syncing() {
    let args = vec!["get", "syncing", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {"
        Success: Syncing status retrieved

        Status: Not syncing
    "});
}

#[tokio::test]
async fn test_json_output_not_syncing() {
    let args = vec!["get", "syncing", "--url", URL, "--json"];
    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();
    let res: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(
        res,
        json!({
            "command": "get syncing",
            "type": "response",
            "syncing": false
        })
    );
}

async fn mock_syncing_server() -> MockServer {
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
        .and(body_partial_json(json!({"method": "starknet_syncing"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": {
                "starting_block_hash": "0x1",
                "starting_block_num": 1,
                "current_block_hash": "0x2",
                "current_block_num": 2,
                "highest_block_hash": "0x3",
                "highest_block_num": 3
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    mock_server
}

#[tokio::test]
async fn test_pretty_output_syncing() {
    let mock_server = mock_syncing_server().await;
    let mock_server_uri = mock_server.uri();

    let args = vec!["get", "syncing", "--url", &mock_server_uri];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {"
        Success: Syncing status retrieved

        Status:                Syncing
        Starting Block Hash:   0x0000000000000000000000000000000000000000000000000000000000000001
        Starting Block Number: 1
        Current Block Hash:    0x0000000000000000000000000000000000000000000000000000000000000002
        Current Block Number:  2
        Highest Block Hash:    0x0000000000000000000000000000000000000000000000000000000000000003
        Highest Block Number:  3
    "});
}

#[tokio::test]
async fn test_json_output_syncing() {
    let mock_rpc = mock_syncing_server().await;
    let mock_rpc_uri = mock_rpc.uri();

    let args = vec!["get", "syncing", "--url", &mock_rpc_uri, "--json"];
    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();
    let res: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(
        res,
        json!({
            "command": "get syncing",
            "type": "response",
            "syncing": true,
            "starting_block_hash": "0x1",
            "starting_block_num": 1,
            "current_block_hash": "0x2",
            "current_block_num": 2,
            "highest_block_hash": "0x3",
            "highest_block_num": 3,
        })
    );
}
