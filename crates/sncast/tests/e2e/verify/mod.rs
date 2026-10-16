mod starkloupe;
mod voyager;

use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const VERIFY_ARGS: &[&str] = &[
    "--accounts-file",
    ACCOUNT_FILE_PATH,
    "verify",
    "--class-hash",
    MAP_CONTRACT_CLASS_HASH_SEPOLIA,
    "--contract-name",
    "Map",
    "--verifier",
    "starkloupe",
    "--network",
    "sepolia",
];

async fn setup_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("content-type", "text/plain")
                .set_body_string("Contract successfully verified"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    mock_server
}

#[tokio::test]
async fn test_verification_confirmation_accepts_empty_input() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let mock_server = setup_mock_server().await;

    let output = runner(VERIFY_ARGS)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("\n")
        .assert()
        .success();

    assert_stdout_contains(output, "Success: Verification completed");
}

#[tokio::test]
async fn test_verification_confirmation_aborts_on_n() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let output = runner(VERIFY_ARGS)
        .current_dir(contract_path.path())
        .stdin("n\n")
        .assert()
        .failure();

    assert_stderr_contains(output, "Error: Verification aborted");
}

#[tokio::test]
async fn test_verification_confirmation_accepts_y() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let mock_server = setup_mock_server().await;

    let output = runner(VERIFY_ARGS)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("y\n")
        .assert()
        .success();

    assert_stdout_contains(output, "Success: Verification completed");
}

#[tokio::test]
async fn test_verification_confirmation_reprompts_on_other_input() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let mock_server = setup_mock_server().await;

    let output = runner(VERIFY_ARGS)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("maybe\ny\n")
        .assert()
        .success();

    assert_stdout_contains(output, "Success: Verification completed");
}
