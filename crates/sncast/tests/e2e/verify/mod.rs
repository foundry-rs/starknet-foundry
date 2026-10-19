mod starkloupe;
mod voyager;

use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use test_case::test_case;
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

#[test_case("Y")]
#[test_case("y")]
#[test_case("Yes")]
#[test_case("yes")]
#[tokio::test]
async fn test_verification_confirmation_accepts_common_inputs(input: &str) {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let mock_server = setup_mock_server().await;

    let output = runner(VERIFY_ARGS)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin(format!("{input}\n"))
        .assert()
        .success();

    assert_stdout_contains(output, "Success: Verification completed");
}

#[test_case("N")]
#[test_case("n")]
#[test_case("No")]
#[test_case("no")]
#[tokio::test]
async fn test_verification_confirmation_rejects_common_inputs(input: &str) {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");

    let output = runner(VERIFY_ARGS)
        .current_dir(contract_path.path())
        .stdin(format!("{input}\n"))
        .assert()
        .failure();

    assert_stderr_contains(output, "Error: Verification aborted");
}

#[test_case("yay")]
#[test_case("nay")]
#[test_case("maybe")]
#[tokio::test]
async fn test_verification_confirmation_reprompts_on_other_input(input: &str) {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let mock_server = setup_mock_server().await;

    let output = runner(VERIFY_ARGS)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin(format!("{input}\ny\n"))
        .assert()
        .success();

    assert_stdout_contains(
        output.as_stdout().to_string(),
        "Success: Verification completed",
    );
    assert_stderr_contains(
        output.as_stderr().to_string(),
        format!("Could not parse {input} as bool."),
    );
}
