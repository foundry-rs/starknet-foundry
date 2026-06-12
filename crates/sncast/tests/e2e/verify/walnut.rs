use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::{copy_directory_to_tempdir, join_tempdirs};
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};
use serde_json::json;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use wiremock::matchers::{body_partial_json, method, path};
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
        .env("VERIFIER_API_URL", mock_server.uri())
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
        .env("VERIFIER_API_URL", mock_server.uri())
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
async fn test_accepts_full_module_path_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let mock_server = MockServer::start().await;

    let verifier_response = "Contract successfully verified";

    Mock::given(method("POST"))
        .and(path("/v1/sn_sepolia/verify"))
        .and(body_partial_json(json!({
            "contract_name": "HelloStarknet"
        })))
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
        "duplicate_contract_name::first_contract::HelloStarknet",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
        "--confirm-verification",
    ];

    let output = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .assert()
        .success();

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
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().failure();

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
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("Y");

    let output = snapbox.assert().failure();

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
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path()).stdin("n");

    let output = snapbox.assert().failure();

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
async fn test_happy_case_lowercase_y() {
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
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .stdin("y");

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

    let output = snapbox.assert().failure();

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
async fn test_errors_on_ambiguous_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--contract-name",
        "HelloStarknet",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .stdin("Y")
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r#"
        Command: verify
        Error: Found more than one contract named "HelloStarknet" at: duplicate_contract_name::first_contract::HelloStarknet, duplicate_contract_name::second_contract::HelloStarknet
        "#},
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
        .env("VERIFIER_API_URL", mock_server.uri())
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
        .env("VERIFIER_API_URL", mock_server.uri())
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
        .env("VERIFIER_API_URL", mock_server.uri())
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

    let output = snapbox.assert().failure();

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
async fn test_test_files_flag_ignored_with_warning() {
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
        "--test-files",
        "--confirm-verification",
    ];

    let snapbox = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
        [WARNING] The `--test-files` option is ignored for Walnut verifier
        Success: Verification completed

        {}
        ",
            verifier_response
        ),
    );
}

#[tokio::test]
async fn test_happy_case_contract_address_with_alias() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&config_dir, &contract_path);

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
        "@map",
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
        "--confirm-verification",
    ];

    let output = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .assert()
        .success();

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
async fn test_happy_case_class_hash_with_alias() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&config_dir, &contract_path);

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
        "@map-class",
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
        "--confirm-verification",
    ];

    let output = runner(&args)
        .env("VERIFIER_API_URL", mock_server.uri())
        .current_dir(contract_path.path())
        .assert()
        .success();

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
async fn test_unknown_alias_contract_address() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&config_dir, &contract_path);

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--contract-address",
        "@unknown",
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: verify
            Error: Invalid contract address

            Caused by:
                Alias `unknown` not found in config
        "},
    );
}

#[tokio::test]
async fn test_unknown_alias_class_hash() {
    let contract_path = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    join_tempdirs(&config_dir, &contract_path);

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "verify",
        "--class-hash",
        "@unknown",
        "--contract-name",
        "Map",
        "--verifier",
        "walnut",
        "--network",
        "sepolia",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: verify
            Error: Invalid class hash

            Caused by:
                Alias `unknown` not found in config
        "},
    );
}
