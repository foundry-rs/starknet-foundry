use crate::helpers::constants::{
    DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, URL,
};
use crate::helpers::fixtures::create_test_provider;

use camino::Utf8PathBuf;
use shared::rpc::{get_rpc_version, is_expected_version};
use sncast::{check_if_legacy_contract, get_account, get_provider, parse_number};
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
        .contains("RPC url not passed nor found in snfoundry.toml"));
}

#[tokio::test]
async fn test_get_account() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        None,
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
        None,
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Accounts file = tests/data/accounts/nonexistentfile.json does not exist!"));
}

#[tokio::test]
async fn test_get_account_invalid_file() {
    let provider = create_test_provider();
    let account = get_account(
        "user1",
        &Utf8PathBuf::from("tests/data/accounts/invalid_format.json"),
        &provider,
        None,
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
        None,
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account name not passed nor found in snfoundry.toml"));
}

#[tokio::test]
async fn test_get_account_no_user_for_network() {
    let provider = create_test_provider();
    let account = get_account(
        "user100",
        &Utf8PathBuf::from("tests/data/accounts/accounts.json"),
        &provider,
        None,
    )
    .await;
    let err = account.unwrap_err();
    assert!(err
        .to_string()
        .contains("Account = user100 not found under network = alpha-sepolia"));
}

#[tokio::test]
async fn test_get_account_failed_to_convert_field_elements() {
    let provider = create_test_provider();
    let account1 = get_account(
        "with_wrong_private_key",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        None,
    )
    .await;
    let err1 = account1.unwrap_err();
    assert!(err1
        .to_string()
        .contains("Failed to convert private key to FieldElement"));

    let account2 = get_account(
        "with_wrong_address",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        None,
    )
    .await;
    let err2 = account2.unwrap_err();
    assert!(err2
        .to_string()
        .contains("Failed to convert address = address to FieldElement"));

    let account3 = get_account(
        "with_wrong_class_hash",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts.json"),
        &provider,
        None,
    )
    .await;
    let err3 = account3.unwrap_err();
    assert!(err3
        .to_string()
        .contains("Failed to convert class hash = class_hash to FieldElement"));
}

// TODO (#1690): Move this test to the shared crate and execute it for a real node
#[tokio::test]
async fn test_supported_rpc_version_matches_devnet_version() {
    let provider = create_test_provider();
    let devnet_spec_version = get_rpc_version(&provider).await.unwrap();
    assert!(is_expected_version(&devnet_spec_version));
}

#[tokio::test]
async fn test_check_if_legacy_contract_by_class_hash() {
    let provider = create_test_provider();
    let class_hash = parse_number(DEVNET_OZ_CLASS_HASH_CAIRO_0)
        .expect("Failed to parse DEVNET_OZ_CLASS_HASH_CAIRO_0");
    let mock_address = parse_number("0x1").unwrap();
    let is_legacy = check_if_legacy_contract(Some(class_hash), mock_address, &provider)
        .await
        .unwrap();
    assert!(is_legacy);
}

#[tokio::test]
async fn test_check_if_legacy_contract_by_address() {
    let provider = create_test_provider();
    let address = parse_number(DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS)
        .expect("Failed to parse DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS");
    let is_legacy = check_if_legacy_contract(None, address, &provider)
        .await
        .unwrap();
    assert!(!is_legacy);
}
