use crate::helpers::constants::{CONTRACTS_DIR, NETWORK, URL};
use crate::helpers::fixtures::{
    duplicate_and_salt_directory, get_transaction_hash, get_transaction_receipt,
};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::Declare;
use std::fs;
use test_case::test_case;

#[test_case("/v1/map", "_xyz", "user1" ; "when cairo1 contract")]
#[test_case("/v2/map", "_xyz", "user2" ; "when cairo2 contract")]
#[tokio::test]
async fn test_happy_case(contract_path: &str, salt: &str, account: &str) {
    let contract_path =
        duplicate_and_salt_directory(CONTRACTS_DIR.to_string() + contract_path, "put", salt);

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        account,
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("cast"))
        .current_dir(&contract_path)
        .args(args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));

    fs::remove_dir_all(contract_path).unwrap();
}

#[tokio::test]
async fn contract_already_declared() {
    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "Map",
    ];

    let snapbox = Command::new(cargo_bin!("cast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/v1/map")
        .args(args);

    let output = String::from_utf8(snapbox.assert().success().get_output().stderr.clone()).unwrap();

    assert!(output.contains("is already declared"));
}

#[tokio::test]
async fn contract_does_not_exist() {
    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        "../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "nonexistent",
        "--max-fee",
        "999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("cast"))
        .current_dir(CONTRACTS_DIR)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: scarb build returned non-zero exit code: 1
    "#});
}

#[test_case("/v1/map", "_zyx", "user1" ; "when cairo1 contract")]
#[test_case("/v2/map", "_zyx", "user2" ; "when cairo2 contract")]
fn test_too_low_max_fee(contract_path: &str, salt: &str, account: &str) {
    let contract_path =
        duplicate_and_salt_directory(CONTRACTS_DIR.to_string() + contract_path, "put", salt);

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        account,
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
    ];

    let snapbox = Command::new(cargo_bin!("cast"))
        .current_dir(&contract_path)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Transaction has been rejected
    "#});

    fs::remove_dir_all(contract_path).unwrap();
}
