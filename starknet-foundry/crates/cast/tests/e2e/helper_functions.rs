//todo: move to integration
use crate::helpers::constants::{NETWORK, URL};
use crate::helpers::fixtures::create_test_provider;

use camino::Utf8PathBuf;
use cast::{get_account, get_provider};
use std::fs;
use url::ParseError;

#[tokio::test]
async fn test_get_provider() {
    let provider = get_provider(URL, NETWORK);
    assert!(provider.await.is_ok());
}

#[tokio::test]
async fn test_get_provider_invalid_url() {
    let provider = get_provider("what", NETWORK);
    let err = provider.await.unwrap_err();
    assert!(err.is::<ParseError>());
}

#[tokio::test]
async fn test_get_provider_empty_url() {
    let provider = get_provider("", NETWORK);
    let err = provider.await.unwrap_err();
    assert!(err
        .to_string()
        .contains("RPC url not passed nor found in Scarb.toml"));
}

#[tokio::test]
async fn test_get_provider_no_network() {
    let provider = get_provider(URL, "");
    let err = provider.await.unwrap_err();
    assert!(err
        .to_string()
        .contains("Network not passed nor found in Scarb.toml"));
}

#[tokio::test]
async fn test_get_provider_wrong_network() {
    let provider = get_provider(URL, "mainnet");
    let err = provider.await.unwrap_err();
    assert!(err
        .to_string()
        .contains("Networks mismatch: requested network is different than provider network!"));
}

#[test]
fn test_get_account() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        NETWORK,
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
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/nonexistentfile.json"),
        &provider,
        NETWORK,
    );
    let err = account.unwrap_err();
    assert!(err.to_string().contains("No such file or directory"));
}

#[test]
fn test_get_account_invalid_file() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/invalid_format.json"),
        &provider,
        NETWORK,
    );
    let err = account.unwrap_err();
    assert!(err.is::<serde_json::Error>());
}

#[test]
fn test_get_account_wrong_network() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        "mainnet",
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account user1 not found under chain id alpha-mainnet"));
}

#[test]
fn test_get_account_no_network() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        "",
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Network not passed nor found in Scarb.toml"));
}

#[test]
fn test_get_account_no_account() {
    let provider = create_test_provider();
    let account = get_account(
        "",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        NETWORK,
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account name not passed nor found in Scarb.toml"));
}

#[test]
fn test_get_account_no_user_for_network() {
    let provider = create_test_provider();
    let account = get_account(
        "user10",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        NETWORK,
    );
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account user10 not found under chain id alpha-goerli"));
}

#[test]
fn test_get_account_failed_to_convert_field_elements() {
    let provider = create_test_provider();
    let account1 = get_account(
        "with_wrong_private_key",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        NETWORK,
    );
    let err1 = account1.unwrap_err();
    assert!(err1
        .to_string()
        .contains("Failed to convert private key: privatekey to FieldElement"));

    let account2 = get_account(
        "with_wrong_address",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        NETWORK,
    );
    let err2 = account2.unwrap_err();
    assert!(err2
        .to_string()
        .contains("Failed to convert account address: address to FieldElement"));
}
