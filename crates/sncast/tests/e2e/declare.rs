use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_accounts_path, get_transaction_hash, get_transaction_receipt,
};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::Declare;
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(contract_path.path())
        .args(args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn contract_already_declared() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "Map",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: An error occurred in the called contract [..]
    "});
}

#[tokio::test]
async fn wrong_contract_name_passed() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "nonexistent",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Failed to find artifacts in starknet_artifacts.json file[..]
    "});
}

#[test_case("/build_fails", "../../accounts/accounts.json" ; "when wrong cairo contract")]
#[test_case("/", "../accounts/accounts.json" ; "when Scarb.toml does not exist")]
fn scarb_build_fails(contract_path: &str, accounts_file_path: &str) {
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + contract_path)
        .args(args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: declare
        error: Failed to build contracts with Scarb: `scarb` exited with error
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
        "user6",
        "--wait",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(contract_path.path())
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Max fee is smaller than the minimal transaction cost
    "});
}

#[test]
fn scarb_no_sierra_artifact() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/no_sierra")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        [..]Make sure you have enabled sierra code generation in Scarb.toml[..]
    "});
}

#[test]
fn scarb_no_casm_artifact() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/no_casm")
        .args(args);

    assert!(
        String::from_utf8(snapbox.assert().success().get_output().stdout.clone())
            .unwrap()
            .contains("class_hash")
    );
}
