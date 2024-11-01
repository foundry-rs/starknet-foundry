use crate::helpers::constants::{
    DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, URL,
};
use crate::helpers::fixtures::create_test_provider;

use camino::Utf8PathBuf;
use shared::rpc::{get_rpc_version, is_expected_version};
use sncast::{check_if_legacy_contract, get_account, get_provider};
use starknet::accounts::Account;
use starknet::macros::felt;
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
    .await
    .unwrap();

    assert_eq!(account.chain_id(), felt!("0x534e5f5345504f4c4941"));
    assert_eq!(
        account.address(),
        felt!("0xf6ecd22832b7c3713cfa7826ee309ce96a2769833f093795fafa1b8f20c48b")
    );
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
    assert!(err
        .to_string()
        .contains("Failed to parse field `alpha-sepolia.?` in file 'tests/data/accounts/invalid_format.json': expected `,` or `}` at line 8 column 9")
    );
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
        "with_invalid_private_key",
        &Utf8PathBuf::from("tests/data/accounts/faulty_accounts_invalid_felt.json"),
        &provider,
        None,
    )
    .await;
    let err = account1.unwrap_err();

    assert!(err.to_string().contains(
        "Failed to parse field `alpha-sepolia.with_invalid_private_key.private_key` in file 'tests/data/accounts/faulty_accounts_invalid_felt.json': expected hex string to be prefixed by '0x' at line 4 column 40"
    ));
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
    let class_hash = DEVNET_OZ_CLASS_HASH_CAIRO_0
        .parse()
        .expect("Failed to parse DEVNET_OZ_CLASS_HASH_CAIRO_0");
    let mock_address = "0x1".parse().unwrap();
    let is_legacy = check_if_legacy_contract(Some(class_hash), mock_address, &provider)
        .await
        .unwrap();
    assert!(is_legacy);
}

#[tokio::test]
async fn test_check_if_legacy_contract_by_address() {
    let provider = create_test_provider();
    let address = DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS
        .parse()
        .expect("Failed to parse DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS");
    let is_legacy = check_if_legacy_contract(None, address, &provider)
        .await
        .unwrap();
    assert!(!is_legacy);
}
