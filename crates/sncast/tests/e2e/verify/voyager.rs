use std::sync::LazyLock;

use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::formatdoc;
use serde_json::{Value, json};
use shared::consts::EXPECTED_RPC_VERSION;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use starknet_types_core::felt::Felt;
use wiremock::matchers::{body_json, body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static SPEC_REQUEST: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_specVersion",
        "params": [],
    })
});
static SPEC_RESPONSE: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": EXPECTED_RPC_VERSION,
    })
});

#[tokio::test]
async fn test_happy_case_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "Map",
        "package_name": "map",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
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
        "--rpc",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc! {"
        command: verify
        message: Map submitted for verification, you can query the status at: {}/class-verify/job/{}
        ",
        mock_server.uri(),
        job_id,
        },
    );
}

#[tokio::test]
async fn test_happy_case_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
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
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(contract_not_found))
        .expect(0)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "Map",
        "package_name": "map",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
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
        "--rpc",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc! {"
        command: verify
        message: Map submitted for verification, you can query the status at: {}/class-verify/job/{}
        ",
        mock_server.uri(),
        job_id,
        },
    );
}

#[tokio::test]
async fn test_happy_case_with_confirm_verification_flag() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "Map",
        "package_name": "map",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
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
        "--confirm-verification",
        "--rpc",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc! {"
        command: verify
        message: Map submitted for verification, you can query the status at: {}/class-verify/job/{}
        ",
        mock_server.uri(),
        job_id,
        },
    );
}

#[tokio::test]
async fn test_failed_verification_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let error = "some error message";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "Map",
        "package_name": "map",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
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
        "--rpc",
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
        command: verify
        error: some error message
        ",
        },
    );
}

#[tokio::test]
async fn test_failed_verification_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
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
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(contract_not_found))
        .expect(0)
        .mount(&mock_rpc)
        .await;

    let error = "some error message";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

    let expected_body = json!({
        "project_dir_path": ".",
        "name": "Map",
        "package_name": "map",
        "license": null
    });
    Mock::given(method("POST"))
        .and(path(format!("class-verify/{class_hash:#068x}")))
        .and(body_partial_json(&expected_body))
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
        "--rpc",
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
        command: verify
        error: some error message
        ",
        },
    );
}

#[tokio::test]
async fn test_failed_class_hash_lookup() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });

    // The official StarkNet error response format
    let contract_not_found = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": 20,
            "message": "Contract not found"
        }
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();

    // Set up the spec version request mock
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    // Set up the class hash lookup request to return the contract not found error
    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(contract_not_found))
        .expect(1)
        .mount(&mock_rpc)
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
        "--confirm-verification",
        "--rpc",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc! {"
        command: verify
        error: ContractNotFound
        ",
        },
    );
}

#[tokio::test]
async fn test_virtual_workspaces() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": {
            "block_id": "latest",
            "contract_address": MAP_CONTRACT_ADDRESS_SEPOLIA
        }
    });
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

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
        "--rpc",
        &mock_rpc_uri,
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc! {"
        command: verify
        message: FibonacciContract submitted for verification, you can query the status at: {}/class-verify/job/{}
        ",
        mock_server.uri(),
        job_id,
        },
    );
}

#[tokio::test]
async fn test_contract_name_not_found() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;
    let rpc_request = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "starknet_getClassHashAt",
        "params": [
            "latest",
            MAP_CONTRACT_ADDRESS_SEPOLIA
        ]
    });
    let rpc_response = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "result": MAP_CONTRACT_CLASS_HASH_SEPOLIA
    });

    let mock_rpc = MockServer::start().await;
    let mock_rpc_uri = mock_rpc.uri().clone();
    Mock::given(method("POST"))
        .and(body_json(LazyLock::force(&SPEC_REQUEST)))
        .respond_with(ResponseTemplate::new(200).set_body_json(LazyLock::force(&SPEC_RESPONSE)))
        .expect(1)
        .mount(&mock_rpc)
        .await;

    Mock::given(method("POST"))
        .and(body_json(rpc_request))
        .respond_with(ResponseTemplate::new(200).set_body_json(rpc_response))
        .expect(0)
        .mount(&mock_rpc)
        .await;

    let job_id = "2b206064-ffee-4955-8a86-1ff3b854416a";
    let class_hash: Felt =
        Felt::from_hex(MAP_CONTRACT_CLASS_HASH_SEPOLIA).expect("Invalid class hash");

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
        "--rpc",
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
        command: verify
        error: Contract named 'non_existent' was not found
        ",
        },
    );
}
