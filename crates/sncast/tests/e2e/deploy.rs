use crate::helpers::constants::{
    ACCOUNT, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA,
};
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use starknet::core::types::TransactionReceipt::Deploy;
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
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[tokio::test]
async fn test_happy_case_with_constructor() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user4",
        "--int-format",
        "--json",
        "deploy",
        "--fee-token",
        "eth",
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[test]
fn test_wrong_calldata() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user9",
        "deploy",
        "--fee-token",
        "eth",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--constructor-calldata",
        "0x1 0x1",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: An error occurred in the called contract[..]Failed to deserialize param #2[..]
        "},
    );
}

#[tokio::test]
async fn test_contract_not_declared() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        ACCOUNT,
        "deploy",
        "--class-hash",
        "0x1",
        "--fee-token",
        "eth",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: An error occurred in the called contract[..]Class with hash[..]is not declared[..]
        "},
    );
}

#[test]
fn test_contract_already_deployed() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user1",
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
        "--fee-token",
        "eth",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: An error occurred [..]Requested ContractAddress[..]is unavailable for deployment[..]
        "},
    );
}

#[test]
fn test_too_low_max_fee() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user7",
        "--wait",
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
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
        command: deploy
        error: Max fee is smaller than the minimal transaction cost
        "},
    );
}
