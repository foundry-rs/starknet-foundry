use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, duplicate_contract_directory_with_salt, get_accounts_path,
    get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use configuration::CONFIG_FILENAME;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use starknet::core::types::TransactionReceipt::Declare;
use std::fs;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case(account: &str) {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", account);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        account,
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_happy_case_specify_package() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "supercomplexcode",
        "--package",
        "main_workspace",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_contract_already_declared() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "Map",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: An error occurred [..]Class with hash[..]is already declared[..]
        "},
    );
}

#[tokio::test]
async fn test_invalid_nonce() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "1123");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--nonce",
        "12345",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Invalid transaction nonce
        "},
    );
}

#[tokio::test]
async fn test_wrong_contract_name_passed() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "nonexistent",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Failed to find nonexistent artifact in starknet_artifacts.json file[..]
        "},
    );
}

#[test]
fn test_scarb_build_fails_when_wrong_cairo_path() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/build_fails");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        "Failed to build contract: Failed to build using scarb; `scarb` exited with error",
    );
}

#[should_panic(expected = "Path to Scarb.toml manifest does not exist")]
#[test]
fn test_scarb_build_fails_scarb_toml_does_not_exist() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();
}

#[test]
fn test_scarb_build_fails_manifest_does_not_exist() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Path to Scarb.toml manifest does not exist =[..]
        "},
    );
}

#[test]
fn test_too_low_max_fee() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "2");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "--wait",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Max fee is smaller than the minimal transaction cost
        "},
    );
}

#[should_panic(expected = "Make sure you have enabled sierra code generation in Scarb.toml")]
#[test]
fn test_scarb_no_sierra_artifact() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/no_sierra");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
        "--fee-token",
        "eth",
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();
}

#[test]
fn test_scarb_no_casm_artifact() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/no_casm");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: declare
        class_hash: [..]
        transaction_hash: [..]
        "},
    );
}

#[tokio::test]
async fn test_many_packages_default() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "supercomplexcode2",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: More than one package found in scarb metadata - specify package using --package flag",
    );
}

#[tokio::test]
async fn test_worskpaces_package_specified_virtual_fibonacci() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--package",
        "cast_fibonacci",
        "--contract-name",
        "FibonacciContract",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success().get_output().clone();
    let output = output.stdout.clone();
    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;
    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_worskpaces_package_no_contract() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "declare",
        "--package",
        "cast_addition",
        "--contract-name",
        "whatever",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Failed to find whatever artifact in starknet_artifacts.json file[..]
        "},
    );
}

#[tokio::test]
async fn test_no_scarb_profile() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "69");
    fs::copy(
        "tests/data/files/correct_snfoundry.toml",
        contract_path.path().join(CONFIG_FILENAME),
    )
    .expect("Failed to copy config file to temp dir");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--profile",
        "profile5",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [WARNING] Profile profile5 does not exist in scarb, using default 'dev' profile.
        command: declare
        class_hash: [..]
        transaction_hash: [..]
    "});
}
