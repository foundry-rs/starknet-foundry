use crate::helpers::{
    constants::{ACCOUNT, ACCOUNT_FILE_PATH, DECLARE_TRANSACTION_HASH, MAP_CLASS_HASH_V1, NETWORK},
    fixtures::create_test_provider,
};
use camino::Utf8PathBuf;
use cast::{get_account, parse_number, wait_for_tx};
use starknet::contract::ContractFactory;
use starknet::core::types::FieldElement;

#[tokio::test]
async fn test_happy_path() {
    let provider = create_test_provider();
    let res = wait_for_tx(&provider, parse_number(DECLARE_TRANSACTION_HASH).unwrap()).await;

    assert!(matches!(res, Ok(_)));
    assert!(matches!(res.unwrap(), "Transaction accepted"));
}

#[tokio::test]
async fn test_rejected_transaction() {
    let provider = create_test_provider();
    let account = get_account(
        ACCOUNT,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        NETWORK,
    )
    .expect("Could not get the account");

    let factory = ContractFactory::new(parse_number(MAP_CLASS_HASH_V1).unwrap(), account);
    let deployment = factory
        .deploy(&Vec::new(), FieldElement::ONE, false)
        .max_fee(FieldElement::ONE);
    let resp = deployment.send().await.unwrap();

    let result = wait_for_tx(&provider, resp.transaction_hash).await;

    assert!(
        matches!(result, Err(message) if message.to_string() == "Transaction has been rejected")
    );
}

#[tokio::test]
#[should_panic(expected = "Could not get transaction with hash: 0x123456789")]
async fn test_wait_for_nonexistent_tx() {
    let provider = create_test_provider();
    wait_for_tx(
        &provider,
        parse_number("0x123456789").expect("Could not parse a number"),
    )
    .await
    .unwrap();
}
