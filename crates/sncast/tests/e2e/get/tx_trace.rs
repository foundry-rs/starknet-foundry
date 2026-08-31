use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use serde_json::{Value, json};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

const MOCK_TRANSACTION_HASH: &str = "0xabc";
const INVOKE_TX_HASH: &str = "0x26476da48e56e5e7025543ad0bb9105df00ee08571c6d17c4207462ff7717c4";
const DECLARE_TX_HASH: &str = "0x6054540622d534ffffb162a0e80c21bc106581eafeb3efad29385b78e04983d";
const DEPLOY_ACCOUNT_TX_HASH: &str =
    "0x06718b783a0b888f5421c4eb76a532feb9fd5167b2b09274298f79798c782b32";
const L1_HANDLER_TX_HASH: &str = "0x4c8c57b3ab646ef56aef3def69a01bc86d049b98f25ebfe3699334d86c24d5";
const REVERTED_INVOKE_TX_HASH: &str =
    "0x00fecca6a328dd11f40b79c30fe22d23bc6975d1a0923a95b90aff4016a84333";

fn invocation() -> Value {
    json!({
        "contract_address": "0x123",
        "entry_point_selector": "0x240060cdb34fcc260f41eac7474ee1d7c80b7e3607daff9ac67c7ea2ebb1c44",
        "calldata": ["0x7"],
        "caller_address": "0x0",
        "class_hash": "0x456",
        "entry_point_type": "EXTERNAL",
        "call_type": "CALL",
        "result": ["0x9"],
        "calls": [],
        "events": [],
        "messages": [],
        "execution_resources": { "l1_gas": 0, "l2_gas": 0 },
        "is_reverted": false
    })
}

fn trace() -> Value {
    json!({
        "type": "INVOKE",
        "validate_invocation": null,
        "execute_invocation": invocation(),
        "fee_transfer_invocation": null,
        "state_diff": null,
        "execution_resources": { "l1_gas": 0, "l1_data_gas": 0, "l2_gas": 0 }
    })
}

async fn mock_trace(mock_server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({ "method": "starknet_specVersion" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0.10.0"
        })))
        .expect(1)
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({
            "method": "starknet_traceTransaction",
            "params": { "transaction_hash": MOCK_TRANSACTION_HASH }
        })))
        .respond_with(response)
        .expect(1)
        .mount(mock_server)
        .await;
}

async fn mock_contract_class(mock_server: &MockServer, abi: Value) {
    Mock::given(method("POST"))
        .and(body_partial_json(json!({ "method": "starknet_getClass" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "sierra_program": [],
                "contract_class_version": "0.1.0",
                "entry_points_by_type": {
                    "CONSTRUCTOR": [],
                    "EXTERNAL": [],
                    "L1_HANDLER": []
                },
                "abi": abi.to_string()
            }
        })))
        .expect(1)
        .mount(mock_server)
        .await;
}

#[tokio::test]
async fn test_invoke_transaction_trace() {
    let args = &["get", "tx-trace", INVOKE_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_invoke_transaction_trace_full() {
    let args = &["get", "tx-trace", INVOKE_TX_HASH, "--full", "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_declare_transaction_trace() {
    let args = &["get", "tx-trace", DECLARE_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_declare_transaction_trace_full() {
    let args = &["get", "tx-trace", DECLARE_TX_HASH, "--full", "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_deploy_account_transaction_trace() {
    let args = &["get", "tx-trace", DEPLOY_ACCOUNT_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_deploy_account_transaction_trace_full() {
    let args = &[
        "get",
        "tx-trace",
        DEPLOY_ACCOUNT_TX_HASH,
        "--full",
        "--url",
        URL,
    ];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_l1_handler_transaction_trace() {
    let args = &["get", "tx-trace", L1_HANDLER_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_l1_handler_transaction_trace_full() {
    let args = &[
        "get",
        "tx-trace",
        L1_HANDLER_TX_HASH,
        "--full",
        "--url",
        URL,
    ];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_reverted_invoke_transaction_trace() {
    let args = &["get", "tx-trace", REVERTED_INVOKE_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_json() {
    let args = &["--json", "get", "tx-trace", INVOKE_TX_HASH, "--url", URL];
    let output = runner(args).assert().success().stderr_eq("");

    insta::assert_snapshot!(output.as_stdout());

    let json: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let trace = &json;

    assert_eq!(json["command"], "get tx-trace");
    assert_eq!(trace["type"], "INVOKE");

    assert_eq!(
        trace["validate_invocation"]["contract_address"],
        "0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1"
    );
    assert_eq!(
        trace["validate_invocation"]["entry_point_selector"],
        "__validate__"
    );
    assert!(
        trace["validate_invocation"]["calldata"]
            .as_str()
            .unwrap()
            .starts_with("array![Call {")
    );
    assert_eq!(
        trace["validate_invocation"]["result"],
        "success: 0x56414c4944"
    );

    assert_eq!(
        trace["execute_invocation"]["contract_address"],
        "0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1"
    );
    assert_eq!(
        trace["execute_invocation"]["entry_point_selector"],
        "__execute__"
    );
    assert_eq!(
        trace["execute_invocation"]["result"],
        "success: array![array![].span()]"
    );
    assert_eq!(
        trace["execute_invocation"]["calls"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        trace["execute_invocation"]["calls"][0]["contract_address"],
        "0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0"
    );
    assert_eq!(
        trace["execute_invocation"]["calls"][0]["entry_point_selector"],
        "transmit"
    );
    assert!(
        trace["execute_invocation"]["calls"][0]["calldata"]
            .as_str()
            .unwrap()
            .starts_with("ReportContext {")
    );
    assert_eq!(trace["execute_invocation"]["calls"][0]["result"], "success");
    assert_eq!(
        trace["execute_invocation"]["calls"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    assert_eq!(trace["execution_resources"]["l1_gas"], 30);
    assert_eq!(trace["execution_resources"]["l1_data_gas"], 576);
    assert_eq!(trace["execution_resources"]["l2_gas"], 0);
    assert_eq!(
        trace["fee_transfer_invocation"]["contract_address"],
        "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
    );
    assert_eq!(
        trace["fee_transfer_invocation"]["entry_point_selector"],
        "transfer"
    );
    assert_eq!(
        trace["fee_transfer_invocation"]["calldata"],
        "ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 1945769550285990_u256"
    );
    assert_eq!(trace["fee_transfer_invocation"]["result"], "success: true");
}

#[tokio::test]
async fn test_full_and_json_conflict() {
    let args = [
        "--json",
        "get",
        "tx-trace",
        MOCK_TRANSACTION_HASH,
        "--full",
        "--url",
        URL,
    ];

    let output = runner(&args).assert().failure();
    assert_stderr_contains(output, "[..]`--full` cannot be used with `--json`[..]");
}

#[tokio::test]
async fn test_alias() {
    let args = &["get", "transaction-trace", DECLARE_TX_HASH, "--url", URL];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_falls_back_when_class_is_unavailable() {
    let mock_server = MockServer::start().await;
    let mut nested_invocation = invocation();
    nested_invocation["contract_address"] = json!("0x789");
    let mut trace = trace();
    trace["execute_invocation"]["calls"] = json!([nested_invocation]);
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": trace
        })),
    )
    .await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({ "method": "starknet_getClass" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": 28, "message": "Class hash not found" }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = &[
        "get",
        "tx-trace",
        MOCK_TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_warns_when_fetched_abi_cannot_decode_trace() {
    let mock_server = MockServer::start().await;
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": trace()
        })),
    )
    .await;
    mock_contract_class(&mock_server, json!([])).await;

    let args = &[
        "get",
        "tx-trace",
        MOCK_TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ];
    let output = runner(args).assert().success();

    insta::assert_snapshot!(output.as_stdout());
}

#[tokio::test]
async fn test_json_includes_abi_decoding_warnings() {
    let mock_server = MockServer::start().await;
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": trace()
        })),
    )
    .await;
    mock_contract_class(&mock_server, json!([])).await;

    let args = &[
        "--json",
        "get",
        "tx-trace",
        MOCK_TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ];
    let output = runner(args).assert().success().stderr_eq("");
    let json: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();

    assert_eq!(
        json["decoding_warnings"],
        json!([
            {
                "reason": "selector_not_found",
                "class_hash": "0x456"
            },
            {
                "reason": "calldata_decoding_failed",
                "class_hash": "0x456"
            },
            {
                "reason": "result_decoding_failed",
                "class_hash": "0x456"
            }
        ])
    );
}

#[tokio::test]
async fn test_transaction_not_found() {
    let args = &["get", "tx-trace", "0x123", "--url", URL];
    let output = runner(args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: get tx-trace
            Error: Transaction with provided hash was not found (does not exist)
        "},
    );
}

#[tokio::test]
async fn test_trace_not_available() {
    let mock_server = MockServer::start().await;
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 10,
                "message": "No trace available for transaction",
                "data": { "status": "RECEIVED" }
            }
        })),
    )
    .await;

    let args = &[
        "get",
        "tx-trace",
        MOCK_TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ];
    let output = runner(args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: get tx-trace
            Error: No trace is available for the transaction (status: Received)
        "},
    );
}
