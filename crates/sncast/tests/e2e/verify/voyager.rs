use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA,
    MAP_CONTRACT_CLASS_HASH_SEPOLIA, devnet_url,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::formatdoc;
use serde_json::json;
use shared::test_utils::output_assert::assert_stderr_contains;
use starknet_types_core::felt::Felt;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

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

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#064x}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error": error })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error": error })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(400))
        .expect(0)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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

    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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
async fn test_contract_name_not_found() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;
    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // For this test, the error happens before any RPC calls are made (contract name not found)
    // So no RPC mocks needed

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash = Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "FibonacciContract",
        "package_name": "cast_fibonacci",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(0)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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

    let url = devnet_url();
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

    // Mock the verification request and verify that test files are included
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#064x}")))
        .and(body_partial_json(json!({
            "name": "Map",
            "package_name": "map"
        })))
        .and(|req: &Request| {
            if let Ok(body_str) = std::str::from_utf8(&req.body)
                && let Ok(body_json) = serde_json::from_str::<serde_json::Value>(body_str)
                && let Some(files) = body_json.get("files")
                && let Some(files_obj) = files.as_object()
            {
                // Verify that test files ARE present
                return files_obj.contains_key("src/test_helpers.cairo")
                    && files_obj.contains_key("src/tests.cairo");
            }
            false
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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

    // Mock the verification request - without --test-files flag, test files should be excluded
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#064x}")))
        .and(body_partial_json(json!({
            "name": "Map",
            "package_name": "map"
        })))
        .and(|req: &Request| {
            if let Ok(body_str) = std::str::from_utf8(&req.body)
                && let Ok(body_json) = serde_json::from_str::<serde_json::Value>(body_str)
                && let Some(files) = body_json.get("files")
                && let Some(files_obj) = files.as_object()
            {
                // Verify that test files are NOT present
                return !files_obj.contains_key("src/test_helpers.cairo")
                    && !files_obj.contains_key("src/tests.cairo");
            }
            false
        })
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "job_id": job_id })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = devnet_url();
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
