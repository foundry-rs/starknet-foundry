use crate::helpers::constants::URL;
use crate::helpers::fixtures::create_test_provider;

use camino::Utf8PathBuf;
use cast::{get_account_from_accounts_file, get_provider};
use starknet::core::chain_id;
use starknet::core::types::FieldElement;
use std::fs;
use url::ParseError;

#[tokio::test]
async fn test_get_provider() {
    let provider = get_provider(URL);
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_get_provider_invalid_url() {
    let provider = get_provider("what");
    let err = provider.unwrap_err();
    assert!(err.is::<ParseError>());
}

#[tokio::test]
async fn test_get_provider_empty_url() {
    let provider = get_provider("");
    let err = provider.unwrap_err();
    assert!(err
        .to_string()
        .contains("RPC url not passed nor found in Scarb.toml"));
}

#[test]
fn test_get_account() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        chain_id::TESTNET,
    );

    assert!(account.is_ok());

    let expected = fs::read_to_string("tests/data/accounts/user1_representation")
        .expect("Failed to read expected debug representation");
    let returned = format!("{:?}", account.unwrap());
    assert_eq!(returned.trim(), expected.trim());
}

#[test]
fn test_get_account_no_file() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/nonexistentfile.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err = account.unwrap_err();
    assert!(err.to_string().contains("No such file or directory"));
}

#[test]
fn test_get_account_invalid_file() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/invalid_format.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err = account.unwrap_err();
    assert!(err.is::<serde_json::Error>());
}

#[test]
fn test_get_account_wrong_chain_id() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        FieldElement::from_hex_be("0x435553544f4d5f434841494e5f4944")
            .expect("Should convert from hex"),
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account user1 not found under network CUSTOM_CHAIN_ID"));
}

#[test]
fn test_get_account_no_account() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account name not passed nor found in Scarb.toml"));
}

#[test]
fn test_get_account_no_user_for_network() {
    let provider = create_test_provider();
    let account = get_account_from_accounts_file(
        "user10",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account user10 not found under network alpha-goerli"));
}

#[test]
fn test_get_account_failed_to_convert_field_elements() {
    let provider = create_test_provider();
    let account1 = get_account_from_accounts_file(
        "with_wrong_private_key",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err1 = account1.unwrap_err();
    assert!(err1
        .to_string()
        .contains("Failed to convert private key: privatekey to FieldElement"));

    let account2 = get_account_from_accounts_file(
        "with_wrong_address",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        chain_id::TESTNET,
    );
    let err2 = account2.unwrap_err();
    assert!(err2
        .to_string()
        .contains("Failed to convert account address: address to FieldElement"));
}
