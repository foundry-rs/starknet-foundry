use crate::helpers::constants::{CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA};
use crate::helpers::fixtures::{copy_directory_to_tempdir, default_cli_args};
use crate::helpers::runner::runner;
use indoc::formatdoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_happy_case() {
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

    let mut args = default_cli_args();
    args.append(&mut vec![
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ]);

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

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

    let mut args = default_cli_args();
    args.append(&mut vec![
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ]);

    let snapbox = runner(&args)
        .env("WALNUT_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

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

    let mut args = default_cli_args();
    args.append(&mut vec![
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "nonexistent",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ]);

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

    let mut args = default_cli_args();
    args.append(&mut vec![
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "nonexistent",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ]);

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
