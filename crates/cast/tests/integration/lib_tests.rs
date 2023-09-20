use crate::helpers::constants::URL;
use crate::helpers::fixtures::create_test_provider;

use camino::Utf8PathBuf;
use cast::{get_account, get_provider};
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

#[tokio::test]
async fn test_get_account() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;

    assert!(account.is_ok());

    let expected = fs::read_to_string("tests/data/accounts/user1_representation")
        .expect("Failed to read expected debug representation");
    let returned = format!("{:?}", account.unwrap());
    assert_eq!(returned.trim(), expected.trim());
}

#[tokio::test]
async fn test_get_account_no_file() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/nonexistentfile.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Accounts file tests/data/accounts/nonexistentfile.json does not exist!"));
}

#[tokio::test]
async fn test_get_account_invalid_file() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/invalid_format.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err = account.unwrap_err();
    assert!(err.is::<serde_json::Error>());
}

#[tokio::test]
async fn test_get_account_no_account() {
    let provider = create_test_provider();
    let account = get_account(
        "",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account name not passed nor found in Scarb.toml"));
}

#[tokio::test]
async fn test_get_account_no_user_for_network() {
    let provider = create_test_provider();
    let account = get_account(
        "user10",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account user10 not found under network alpha-goerli"));
}

#[tokio::test]
async fn test_get_account_failed_to_convert_field_elements() {
    let provider = create_test_provider();
    let account1 = get_account(
        "with_wrong_private_key",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err1 = account1.unwrap_err();
    assert!(err1
        .to_string()
        .contains("Failed to convert private key: privatekey to FieldElement"));

    let account2 = get_account(
        "with_wrong_address",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        &Utf8PathBuf::default(),
    )
    .await;
    let err2 = account2.unwrap_err();
    assert!(err2
        .to_string()
        .contains("Failed to convert account address: address to FieldElement"));
}
