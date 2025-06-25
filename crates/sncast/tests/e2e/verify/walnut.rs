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
async fn test_happy_case_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        Success: Verification completed

        {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_happy_case_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        Success: Verification completed

        {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_failed_verification_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;

    let verifier_response = "An error occurred during verification: contract class isn't declared";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(400)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        Command: verify
        Error: {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_failed_verification_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;

    let verifier_response = "An error occurred during verification: contract class isn't declared";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(400)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        Command: verify
        Error: {}
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
        "--contract-name",
        "nonexistent",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("n");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        Command: verify
        Error: Verification aborted
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
        "--contract-name",
        "nonexistent",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("Y");

    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
        Command: verify
        Error: Contract named 'nonexistent' was not found
        "
        ),
    );
}

#[tokio::test]
async fn test_happy_case_with_confirm_verification_flag() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "walnut",
        "--network",
        "sepolia",
        "--confirm-verification",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        Success: Verification completed

        {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_happy_case_specify_package() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
        .mount(&mock_server)
        .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "supercomplexcode",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
        "--package",
        "main_workspace",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(tempdir.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        Success: Verification completed

        {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_worskpaces_package_specified_virtual_fibonacci() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string(verifier_response),
        )
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
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
        "--package",
        "cast_fibonacci",
    ];

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(tempdir.path())
        .stdin("Y");

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        Success: Verification completed

        {}
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
        "--contract-name",
        "nonexistent",
        "--verifier",
        "walnut",
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
        Command: verify
        Error: Contract named 'nonexistent' was not found
        "
        ),
    );
}
