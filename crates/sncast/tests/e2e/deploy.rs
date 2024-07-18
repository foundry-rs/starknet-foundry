use crate::helpers::constants::{
    ACCOUNT, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, DEVNET_OZ_CLASS_HASH_CAIRO_0,
    MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::fixtures::{
    create_and_deploy_account, create_and_deploy_oz_account, default_cli_args,
    get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use sncast::helpers::constants::{ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH};
use sncast::AccountType;
use starknet::core::types::TransactionReceipt::Deploy;
use starknet_crypto::FieldElement;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case_eth(account: &str) {
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

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
#[test_case(BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case_strk(class_hash: FieldElement, account_type: AccountType) {
    let tempdir = create_and_deploy_account(class_hash, account_type).await;
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
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
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let hash = get_transaction_hash(&snapbox.assert().success().get_output().stdout.clone());
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[test_case("v1"; "v1")]
#[test_case("v3"; "v3")]
#[tokio::test]
async fn test_happy_case_versions(version: &str) {
    let tempdir = create_and_deploy_oz_account().await;
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
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
        "--version",
        version,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}
#[test_case(Some("99999999999999999"), None, None; "max_fee")]
#[test_case(None, Some("999"), None; "max_gas")]
#[test_case(None, None, Some("999999999999"); "max_gas_unit_price")]
#[test_case(None, None, None; "none")]
#[test_case(Some("9999999999999"), None, Some("999999999999"); "max_fee_max_gas_unit_price")]
#[test_case(None, Some("999"), Some("999999999999"); "max_gas_max_gas_unit_price")]
#[test_case(Some("999999999999999"), Some("999"), None; "max_fee_max_gas")]
#[tokio::test]
async fn test_happy_case_strk_different_fees(
    max_fee: Option<&str>,
    max_gas: Option<&str>,
    max_gas_unit_price: Option<&str>,
) {
    let tempdir = create_and_deploy_oz_account().await;

    let mut args = vec![
        "--url",
        URL,
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
        "--fee-token",
        "strk",
    ];
    let options = [
        ("--max-fee", max_fee),
        ("--max-gas", max_gas),
        ("--max-gas-unit-price", max_gas_unit_price),
    ];

    for &(key, value) in &options {
        if let Some(val) = value {
            args.append(&mut vec![key, val]);
        }
    }

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();
    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[test_case("eth", "v3"; "eth-v3")]
#[test_case("strk", "v1"; "strk-v3")]
#[tokio::test]
async fn test_invalid_version_and_token_combination(fee_token: &str, version: &str) {
    let tempdir = create_and_deploy_oz_account().await;

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
        "--fee-token",
        fee_token,
        "--version",
        version,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        format!("Error: {fee_token} fee token is not supported for {version} deployment."),
    );
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
