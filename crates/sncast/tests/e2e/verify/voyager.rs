use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::formatdoc;
use serde_json::{Value, json};
use shared::test_utils::output_assert::assert_stderr_contains;
use starknet_types_core::felt::Felt;
use std::fs;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

#[derive(Clone, Copy)]
struct ExpectedVoyagerPayload<'a> {
    name: &'a str,
    package_name: &'a str,
    contract_file: &'a str,
}

fn parse_request_body(req: &Request) -> Option<Value> {
    std::str::from_utf8(&req.body)
        .ok()
        .and_then(|body| serde_json::from_str(body).ok())
}

fn matches_voyager_payload(req: &Request, expected: ExpectedVoyagerPayload<'_>) -> bool {
    let Some(body) = parse_request_body(req) else {
        return false;
    };

    body.get("name").and_then(Value::as_str) == Some(expected.name)
        && body.get("package_name").and_then(Value::as_str) == Some(expected.package_name)
        && body.get("contract_file").and_then(Value::as_str) == Some(expected.contract_file)
        && body.get("contract-name").and_then(Value::as_str) == Some(expected.contract_file)
        && body.get("project_dir_path").and_then(Value::as_str) == Some(".")
        && body.get("license").and_then(Value::as_str) == Some("NONE")
        && body
            .get("files")
            .and_then(Value::as_object)
            .is_some_and(|files| files.contains_key(expected.contract_file))
}

#[tokio::test]
async fn test_happy_case_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Only mock the getClassHashAt call that voyager actually makes
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| matches_voyager_payload(req, expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}

#[tokio::test]
async fn test_happy_case_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // For class hash tests, no RPC calls are made since we already have the class hash
    // No need to mock any RPC calls

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| matches_voyager_payload(req, expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}

#[tokio::test]
async fn test_happy_case_with_confirm_verification_flag() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Only mock the getClassHashAt call that voyager actually makes
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| matches_voyager_payload(req, expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--confirm-verification",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path());

    snapbox.assert().success();
}

#[tokio::test]
async fn test_failed_verification_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Only mock the getClassHashAt call that voyager actually makes
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let error = "some error message";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error": error })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        Command: verify
        Error: {}
        ",
        error,
        },
    );
}

#[tokio::test]
async fn test_failed_verification_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // For class hash tests, no RPC calls are made since we already have the class hash
    // No need to mock any RPC calls

    let error = "some error message";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error": error })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        Command: verify
        Error: {}
        ",
        error,
        },
    );
}

#[tokio::test]
async fn test_failed_class_hash_lookup() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let contract_not_found = json!({
        "error": {
            "code": 20,
            "message": "Contract not found"
        },
        "id": 1,
        "jsonrpc": "2.0"
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Mock the getClassHashAt call to return contract not found
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(400).set_body_json(contract_not_found))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    // Voyager API should not be called since RPC call fails
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .respond_with(ResponseTemplate::new(400))
        .expect(0)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        Command: verify
        Error: ContractNotFound
        ",
        },
    );
}

#[tokio::test]
async fn test_virtual_workspaces() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Only mock the getClassHashAt call that voyager actually makes
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "FibonacciContract",
        package_name: "cast_fibonacci",
        contract_file: "crates/cast_fibonacci/src/lib.cairo",
    };

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| matches_voyager_payload(req, expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "FibonacciContract",
        "--package",
        "cast_fibonacci",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}

#[tokio::test]
async fn test_uploaded_scarb_toml_removes_dev_dependencies() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let manifest_path = contract_path.path().join("Scarb.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("Failed to read test Scarb.toml");
    fs::write(
        &manifest_path,
        format!(
            "{manifest}\n[dev-dependencies]\nsnforge_std = \"0.43.0\"\n\n[scripts]\ncheck = \"scarb check\"\n"
        ),
    )
    .expect("Failed to write test Scarb.toml");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "sanitized-scarb-toml-job";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| {
            if !matches_voyager_payload(req, expected) {
                return false;
            }

            parse_request_body(req)
                .and_then(|body| body.get("files").cloned())
                .and_then(|files| files.get("Scarb.toml").cloned())
                .and_then(|scarb_toml| scarb_toml.as_str().map(ToString::to_string))
                .is_some_and(|scarb_toml| {
                    !scarb_toml.contains("\n[dev-dependencies]")
                        && !scarb_toml.contains("snforge_std")
                        && scarb_toml
                            .contains("# [dev-dependencies] section removed for remote compilation")
                        && scarb_toml.contains("[scripts]")
                })
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}

#[tokio::test]
async fn test_contract_name_not_found() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;
    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // For this test, the error happens before any RPC calls are made (contract name not found)
    // So no RPC mocks needed

    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": "unused" })))
        .expect(0)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "non_existent",
        "--package",
        "cast_fibonacci",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        Command: verify
        Error: Contract named 'non_existent' was not found
        ",
        },
    );
}

#[tokio::test]
async fn test_error_when_neither_network_nor_url_provided() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--confirm-verification",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        Command: verify
        Error: Either --network or --url must be provided
        ",
        },
    );
}

#[tokio::test]
async fn test_test_files_flag_includes_test_files() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Mock the getClassHashAt call
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "test-job-id-with-test-files";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    // Mock the verification request and verify that test files are included
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| {
            if !matches_voyager_payload(req, expected) {
                return false;
            }

            parse_request_body(req)
                .and_then(|body| body.get("files").cloned())
                .and_then(|files| files.as_object().cloned())
                .is_some_and(|files| {
                    files.contains_key("src/test_helpers.cairo")
                        && files.contains_key("src/tests.cairo")
                })
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
        "--test-files",
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}

#[tokio::test]
async fn test_without_test_files_flag_excludes_test_files() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Mock the getClassHashAt call
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "starknet_getClassHashAt"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "test-job-id-without-test-files";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");
    let expected = ExpectedVoyagerPayload {
        name: "Map",
        package_name: "map",
        contract_file: "src/lib.cairo",
    };

    // Mock the verification request - without --test-files flag, test files should be excluded
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#066x}")))
        .and(move |req: &Request| {
            if !matches_voyager_payload(req, expected) {
                return false;
            }

            parse_request_body(req)
                .and_then(|body| body.get("files").cloned())
                .and_then(|files| files.as_object().cloned())
                .is_some_and(|files| {
                    !files.contains_key("src/test_helpers.cairo")
                        && !files.contains_key("src/tests.cairo")
                })
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--url",
        &mock_rpc_uri,
        // Note: --test-files flag is NOT included
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    snapbox.assert().success();
}
