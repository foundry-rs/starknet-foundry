use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_accounts_path, get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Declare;
use std::fs;
use std::path::Path;
use test_case::test_case;

#[tokio::test]
async fn test_happy_case() {
    let contract_path =
        duplicate_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "1");
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
        "Map",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = runner(&args, Some(contract_path.path()));
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));

    fs::remove_dir_all(contract_path).unwrap();
}

#[tokio::test]
async fn contract_already_declared() {
    let contract_path = Path::new(CONTRACTS_DIR).join("map");
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
    ];

    let snapbox = runner(&args, Some(&contract_path));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Class with hash [..] is already declared.
    "});
}

#[tokio::test]
async fn wrong_contract_name_passed() {
    let contract_path = Path::new(CONTRACTS_DIR).join("map");
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
    ];

    let snapbox = runner(&args, Some(&contract_path));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Failed to find artifacts in starknet_artifacts.json file[..]
    "});
}

#[test_case("build_fails", "../../accounts/accounts.json" ; "when wrong cairo contract")]
#[test_case(".", "../accounts/accounts.json" ; "when Scarb.toml does not exist")]
fn scarb_build_fails(relative_contract_path: &str, accounts_file_path: &str) {
    let contract_path = Path::new(CONTRACTS_DIR).join(relative_contract_path);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file_path,
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
    ];
    let snapbox = runner(&args, Some(&contract_path));

    snapbox.assert().stderr_matches(indoc! {r"
        command: declare
        error: Scarb build returned non-zero exit code: 1[..]
        ...
    "});
}

#[test]
fn test_too_low_max_fee() {
    let contract_path =
        duplicate_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "2");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "--wait",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
    ];

    let snapbox = runner(&args, Some(contract_path.path()));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Max fee is smaller than the minimal transaction cost (validation plus fee transfer)
    "});

    fs::remove_dir_all(contract_path).unwrap();
}

#[test_case("no_sierra", "../../accounts/accounts.json" ; "when there is no sierra artifact")]
#[test_case("no_casm", "../../accounts/accounts.json" ; "when there is no casm artifact")]
fn scarb_no_artifacts(relative_contract_path: &str, accounts_file_path: &str) {
    let contract_path = Path::new(CONTRACTS_DIR).join(relative_contract_path);
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file_path,
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    let snapbox = runner(&args, Some(&contract_path));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        [..]Make sure you have enabled sierra and casm code generation in Scarb.toml[..]
    "});
}
