use crate::helpers::{
    constants::{ACCOUNT, ACCOUNT_FILE_PATH},
    fixtures::{create_test_provider, invoke_contract},
};
use sncast::helpers::constants::UDC_ADDRESS;

use crate::helpers::constants::{
    CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
    MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA,
};
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use sncast::{get_account, ValidatedWaitParams};
use sncast::{handle_wait_for_tx, wait_for_tx, WaitForTx};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;

#[tokio::test]
async fn test_happy_path() {
    let provider = create_test_provider();
    let res = wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::default(),
    )
    .await;

    assert!(res.is_ok());
    assert!(matches!(res.unwrap(), "Transaction accepted"));
}

#[tokio::test]
async fn test_rejected_transaction() {
    let provider = create_test_provider();
    let account = get_account(
        ACCOUNT,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        None,
    )
    .await
    .expect("Could not get the account");

    let factory = ContractFactory::new(MAP_CONTRACT_CLASS_HASH_SEPOLIA.parse().unwrap(), account);
    let deployment = factory
        .deploy_v1(Vec::new(), FieldElement::ONE, false)
        .max_fee(FieldElement::ONE);
    let resp = deployment.send().await.unwrap_err();

    assert!(resp.to_string().contains("InsufficientMaxFee"));
}

#[tokio::test]
#[should_panic(expected = "Transaction has been reverted = Insufficient max fee:")]
async fn test_wait_for_reverted_transaction() {
    let provider = create_test_provider();
    let salt = "0x029c81e6487b5f9278faa6f454cda3c8eca259f99877faab823b3704327cd695";
    let max_fee: u64 = 43_400_000_000_000 - 1;

    let transaction_hash = invoke_contract(
        ACCOUNT,
        UDC_ADDRESS.into_hex_string().as_str(),
        "deployContract",
        Some(max_fee.into()),
        &[
            CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
            salt,
            "0x1",
            "0x3",
            "0x43",
            "0x41",
            "0x1",
        ],
    )
    .await
    .transaction_hash;

    wait_for_tx(&provider, transaction_hash, ValidatedWaitParams::new(1, 3))
        .await
        .map_err(std::convert::Into::<anyhow::Error>::into)
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "sncast timed out while waiting for transaction to succeed")]
async fn test_wait_for_nonexistent_tx() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        "0x123456789".parse().expect("Could not parse a number"),
        ValidatedWaitParams::new(1, 3),
    )
    .await
    .map_err(anyhow::Error::from)
    .unwrap();
}

#[tokio::test]
async fn test_happy_path_handle_wait_for_tx() {
    let provider = create_test_provider();
    let res = handle_wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        1,
        WaitForTx {
            wait: true,
            wait_params: ValidatedWaitParams::new(5, 63),
        },
    )
    .await;

    assert!(matches!(res, Ok(1)));
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(2, 1),
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_timeout_zero() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(2, 0),
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_interval_zero() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(0, 1),
    )
    .await
    .unwrap();
}
