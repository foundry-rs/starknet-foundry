use crate::helpers::{
    constants::{ACCOUNT, ACCOUNT_FILE_PATH},
    fixtures::{create_test_provider, from_env, invoke_contract},
};
use sncast::helpers::constants::UDC_ADDRESS;

use camino::Utf8PathBuf;
use sncast::{
    get_account,
    helpers::constants::{WAIT_RETRY_INTERVAL, WAIT_TIMEOUT},
};
use sncast::{handle_wait_for_tx, parse_number, wait_for_tx, WaitForTx};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;

#[tokio::test]
async fn test_happy_path() {
    let provider = create_test_provider();
    let hash = from_env("CAST_MAP_DECLARE_HASH").unwrap();
    let res = wait_for_tx(
        &provider,
        parse_number(&hash).unwrap(),
        WAIT_TIMEOUT,
        WAIT_RETRY_INTERVAL,
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
    let class_hash = from_env("CAST_MAP_CLASS_HASH").unwrap();

    let factory = ContractFactory::new(parse_number(&class_hash).unwrap(), account);
    let deployment = factory
        .deploy(Vec::new(), FieldElement::ONE, false)
        .max_fee(FieldElement::ONE);
    let resp = deployment.send().await.unwrap_err();
    dbg!(&resp);

    assert!(resp.to_string().contains("InsufficientMaxFee"));
}

#[tokio::test]
#[should_panic(expected = "Transaction has been reverted = Insufficient max fee:")]
async fn test_wait_for_reverted_transaction() {
    let provider = create_test_provider();
    let class_hash = from_env("CAST_WITH_CONSTRUCTOR_CLASS_HASH").unwrap();
    let salt = "0x029c81e6487b5f9278faa6f454cda3c8eca259f99877faab823b3704327cd695";
    let max_fee: u64 = 332_323_232_324_342;

    let transaction_hash = invoke_contract(
        ACCOUNT,
        UDC_ADDRESS,
        "deployContract",
        Some(max_fee.into()),
        &[&class_hash, salt, "0x1", "0x3", "0x43", "0x41", "0x1"],
    )
    .await
    .transaction_hash;

    wait_for_tx(&provider, transaction_hash, 3, 1)
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(
    expected = "Failed to get transaction with hash = 0x123456789; Transaction rejected, not received or sncast timed out"
)]
async fn test_wait_for_nonexistent_tx() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        parse_number("0x123456789").expect("Could not parse a number"),
        3,
        1,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_happy_path_handle_wait_for_tx() {
    let provider = create_test_provider();
    let hash = from_env("CAST_MAP_DECLARE_HASH").unwrap();
    let res = handle_wait_for_tx(
        &provider,
        parse_number(&hash).unwrap(),
        1,
        WaitForTx {
            wait: true,
            timeout: 63,
            retry_interval: 5,
        },
    )
    .await;

    assert!(matches!(res, Ok(1)));
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values() {
    let provider = create_test_provider();
    let hash = from_env("CAST_MAP_DECLARE_HASH").unwrap();
    wait_for_tx(&provider, FieldElement::from_dec_str(&hash).unwrap(), 1, 2)
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_timeout_zero() {
    let provider = create_test_provider();
    let hash = from_env("CAST_MAP_DECLARE_HASH").unwrap();
    wait_for_tx(&provider, FieldElement::from_dec_str(&hash).unwrap(), 0, 2)
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(expected = "Invalid values for retry_interval and/or timeout!")]
async fn test_wait_for_wrong_retry_values_interval_zero() {
    let provider = create_test_provider();
    let hash = from_env("CAST_MAP_DECLARE_HASH").unwrap();
    wait_for_tx(&provider, FieldElement::from_dec_str(&hash).unwrap(), 1, 0)
        .await
        .unwrap();
}
