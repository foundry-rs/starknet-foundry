use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::formatdoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_happy_case() {
    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let custom_api_url = &mock_server.uri().clone();

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--custom-base-api-url",
        custom_api_url,
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        command: verify
        message: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_happy_case_class_hash() {
    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let custom_api_url = &mock_server.uri().clone();

    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--custom-base-api-url",
        custom_api_url,
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        command: verify
        message: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_failed_verification() {
    let mock_server = MockServer::start().await;

    let verifier_response = "An error occurred during verification: contract class isn't declared";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(400)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let custom_api_url = &mock_server.uri().clone();

    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--custom-base-api-url",
        custom_api_url,
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        command: verify
        error: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_verification_abort() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "nonexistent",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("n");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        command: verify
        error: Verification aborted
        "
        ),
    );
}

#[tokio::test]
async fn test_wrong_contract_name_passed() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "nonexistent",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        command: verify
        error: Contract named 'nonexistent' was not found
        "
        ),
    );
}

#[tokio::test]
async fn test_no_class_hash_or_contract_address_provided() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
            error: the following required arguments were not provided:
              <--contract-address <CONTRACT_ADDRESS>|--class-hash <CLASS_HASH>>

            Usage: sncast verify --class-name <CLASS_NAME> --network <NETWORK> --verifier <VERIFIER> <--contract-address <CONTRACT_ADDRESS>|--class-hash <CLASS_HASH>>

            For more information, try '--help'."
        ),
    );
}

#[tokio::test]
async fn test_both_class_hash_or_contract_address_provided() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
            error: the argument '--contract-address <CONTRACT_ADDRESS>' cannot be used with '--class-hash <CLASS_HASH>'
            Usage: sncast verify --class-name <CLASS_NAME> --verifier <VERIFIER> --network <NETWORK> <--contract-address <CONTRACT_ADDRESS>|--class-hash <CLASS_HASH>>"
        ),
    );
}

#[tokio::test]
async fn test_happy_case_with_confirm_verification_flag() {
    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let custom_api_url = &mock_server.uri();

    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "Map",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--custom-base-api-url",
        custom_api_url,
        "--confirm-verification",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        command: verify
        message: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_happy_case_specify_package() {
    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let custom_api_url = &mock_server.uri();

    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "supercomplexcode",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--package",
        "main_workspace",
        "--custom-base-api-url",
        custom_api_url,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        command: verify
        message: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_worskpaces_package_specified_virtual_fibonacci() {
    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/class-verify-v2"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let custom_api_url = &mock_server.uri();

    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "FibonacciContract",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--package",
        "cast_fibonacci",
        "--custom-base-api-url",
        custom_api_url,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        command: verify
        message: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_worskpaces_package_no_contract() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--class-name",
        "nonexistent",
        "--verifier",
        "voyager",
        "--network",
        "sepolia",
        "--package",
        "cast_addition",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        command: verify
        error: Contract named 'nonexistent' was not found
        "
        ),
    );
}
