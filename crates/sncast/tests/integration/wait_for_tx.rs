use crate::helpers::{
    constants::{ACCOUNT, ACCOUNT_FILE_PATH},
    fixtures::{create_test_provider, invoke_contract},
};
use foundry_ui::UI;
use sncast::helpers::{constants::UDC_ADDRESS, fee::FeeSettings};

use crate::helpers::constants::{
    CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
    MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA,
};
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use sncast::{ValidatedWaitParams, get_account};
use sncast::{WaitForTx, handle_wait_for_tx, wait_for_tx};
use starknet::contract::ContractFactory;
use starknet_types_core::felt::Felt;

#[tokio::test]
async fn test_happy_path() {
    let provider = create_test_provider();
    let ui = UI::default();
    let res = wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::default(),
        &ui,
    )
    .await;

    assert!(res.is_ok());
    assert!(matches!(res.unwrap().as_str(), "Transaction accepted"));
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
        .deploy_v3(Vec::new(), Felt::ONE, false)
        .l1_gas(1)
        .l2_gas(1)
        .l1_data_gas(1);

    let resp = deployment.send().await.unwrap_err();

    assert!(
        resp.to_string()
            .contains("InsufficientResourcesForValidate")
    );
}

#[tokio::test]
#[should_panic(
    expected = "Transaction execution failed: Provider(StarknetError(InsufficientResourcesForValidate))"
)]
async fn test_wait_for_reverted_transaction() {
    let provider = create_test_provider();
    let salt = "0x029c81e6487b5f9278faa6f454cda3c8eca259f99877faab823b3704327cd695";

    let fee_settings = FeeSettings {
        l1_gas: Some(1),
        l1_gas_price: Some(1),
        l2_gas: Some(1),
        l2_gas_price: Some(1),
        l1_data_gas: Some(1),
        l1_data_gas_price: Some(1),
        tip: None,
    };
    let transaction_hash = invoke_contract(
        ACCOUNT,
        UDC_ADDRESS.into_hex_string().as_str(),
        "deployContract",
        fee_settings,
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

    let ui = UI::default();
    wait_for_tx(
        &provider,
        transaction_hash,
        ValidatedWaitParams::new(1, 3),
        &ui,
    )
    .await
    .map_err(anyhow::Error::from)
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "sncast timed out while waiting for transaction to succeed")]
async fn test_wait_for_nonexistent_tx() {
    let provider = create_test_provider();
    let ui = UI::default();
    wait_for_tx(
        &provider,
        "0x123456789".parse().expect("Could not parse a number"),
        ValidatedWaitParams::new(1, 3),
        &ui,
    )
    .await
    .map_err(anyhow::Error::from)
    .unwrap();
}

#[tokio::test]
async fn test_happy_path_handle_wait_for_tx() {
    let provider = create_test_provider();
    let ui = UI::default();
    let res = handle_wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        1,
        WaitForTx {
            wait: true,
            wait_params: ValidatedWaitParams::new(5, 63),
        },
        &ui,
    )
    .await;

    assert!(matches!(res, Ok(1)));
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values() {
    let provider = create_test_provider();
    let ui = UI::default();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(2, 1),
        &ui,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_timeout_zero() {
    let provider = create_test_provider();
    let ui = UI::default();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(2, 0),
        &ui,
    )
    .await
    .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_interval_zero() {
    let provider = create_test_provider();
    let ui = UI::default();
    wait_for_tx(
        &provider,
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA.parse().unwrap(),
        ValidatedWaitParams::new(0, 1),
        &ui,
    )
    .await
    .unwrap();
}
