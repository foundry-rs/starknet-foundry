use crate::helpers::constants::{ACCOUNT, ACCOUNT_FILE_PATH, MAP_CONTRACT_ADDRESS_SEPOLIA, URL};
use crate::helpers::fixtures::{get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use starknet::core::types::TransactionReceipt::Invoke;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case(account: &str) {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        account,
        "--int-format",
        "--json",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        ACCOUNT,
        "invoke",
        "--url",
        URL,
        "--contract-address",
        "0x1",
        "--function",
        "put",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: invoke
        error: An error occurred in the called contract[..]Requested contract address[..]is not deployed[..]
        "},
    );
}

#[test]
fn test_wrong_function_name() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user2",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nonexistent_put",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: invoke
        error: An error occurred in the called contract[..]Entry point[..]not found in contract[..]
        "},
    );
}

#[test]
fn test_wrong_calldata() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user5",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: invoke
        error: An error occurred in the called contract[..]Failed to deserialize param #2[..]
        "},
    );
}

#[test]
fn test_too_low_max_fee() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user11",
        "--wait",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "0x2",
        "--max-fee",
        "1",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: invoke
        error: Max fee is smaller than the minimal transaction cost
        "},
    );
}
