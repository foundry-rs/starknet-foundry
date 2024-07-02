use crate::helpers::constants::{ACCOUNT, MAP_CONTRACT_ADDRESS_SEPOLIA};
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use starknet::core::types::TransactionReceipt::Invoke;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case(account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--int-format",
        "--json",
        "invoke",
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
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        ACCOUNT,
        "invoke",
        "--contract-address",
        "0x1",
        "--function",
        "put",
        "--fee-token",
        "eth",
    ]);

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
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nonexistent_put",
        "--fee-token",
        "eth",
    ]);

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
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user5",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "--fee-token",
        "eth",
    ]);

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
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user11",
        "--wait",
        "invoke",
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
    ]);

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
