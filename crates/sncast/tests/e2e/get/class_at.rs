use crate::helpers::runner::runner;
use serde_json::json;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mock_server() -> MockServer {
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
        .and(body_partial_json(json!({"method": "starknet_getClassAt"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1,
            "jsonrpc": "2.0",
            "result": {
                "sierra_program": ["0x1"],
                "contract_class_version": "0.1.0",
                "entry_points_by_type": {
                    "CONSTRUCTOR": [],
                    "EXTERNAL": [],
                    "L1_HANDLER": []
                },
                "abi": "[]"
            }
        })))
        .mount(&mock_server)
        .await;

    mock_server
}

#[tokio::test]
async fn test_pretty_output() {
    let mock_server = mock_server().await;
    let url = mock_server.uri();
    let args = vec!["get", "class-at", "0x1", "--url", &url];

    let output = runner(&args).assert().success();

    let res: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let expected = json!({
        "sierra_program": ["0x1"],
        "contract_class_version": "0.1.0",
        "entry_points_by_type": {
            "CONSTRUCTOR": [],
            "EXTERNAL": [],
            "L1_HANDLER": []
        },
        "abi": "[]"
    });

    assert_eq!(res, expected);

    let output = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    assert!(
        output.lines().count() > 1,
        "Expected result to be pretty-printed"
    );
}

#[tokio::test]
async fn test_json_output() {
    let mock_server = mock_server().await;
    let url = mock_server.uri();
    let args = vec!["--json", "get", "class-at", "0x1", "--url", &url];

    let output = runner(&args).assert().success();

    let res: serde_json::Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let expected = json!({
        "type": "response",
        "command": "get class-at",
        "sierra_program": ["0x1"],
        "contract_class_version": "0.1.0",
        "entry_points_by_type": {
            "CONSTRUCTOR": [],
            "EXTERNAL": [],
            "L1_HANDLER": []
        },
        "abi": "[]"
    });

    assert_eq!(res, expected);
}
