use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    create_and_deploy_account, create_and_deploy_oz_account, get_transaction_hash,
    get_transaction_receipt,
};
use crate::helpers::runner::runner;
use crate::helpers::shell::os_specific_shell;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use snapbox::cmd::cargo_bin;
use sncast::AccountType;
use sncast::helpers::constants::{ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH};
use sncast::helpers::fee::FeeArgs;
use starknet::core::types::TransactionReceipt::Invoke;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use test_case::test_case;

#[tokio::test]
async fn test_happy_case_human_readable() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Invocation completed

            Transaction Hash: 0x0[..]

            To see invocation details, visit:
            transaction: [..]
            "
        },
    );
}

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
#[test_case(BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case(class_hash: Felt, account_type: AccountType) {
    let tempdir = create_and_deploy_account(class_hash, account_type).await;
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
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
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let hash = get_transaction_hash(&stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[test_case(FeeArgs{
    max_fee: Some(NonZeroFelt::try_from(Felt::from(1_000_000_000_000_000_000_000_000_u128)).unwrap()),
    l1_data_gas: None,
    l1_data_gas_price:  None,
    l1_gas:  None,
    l1_gas_price:  None,
    l2_gas:  None,
    l2_gas_price:  None,
}; "max_fee")]
#[test_case(FeeArgs{
    max_fee: None,
    l1_data_gas: Some(100_000),
    l1_data_gas_price: Some(10_000_000_000_000),
    l1_gas: Some(100_000),
    l1_gas_price: Some(10_000_000_000_000),
    l2_gas: Some(1_000_000_000),
    l2_gas_price: Some(100_000_000_000_000_000_000),
}; "resource_bounds")]
#[tokio::test]
async fn test_happy_case_different_fees(fee_args: FeeArgs) {
    let tempdir = create_and_deploy_oz_account().await;
    let mut args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
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
    ];
    let options = [
        (
            "--max-fee",
            fee_args.max_fee.map(Felt::from).map(|x| x.to_string()),
        ),
        ("--l1-data-gas", fee_args.l1_data_gas.map(|x| x.to_string())),
        (
            "--l1-data-gas-price",
            fee_args.l1_data_gas_price.map(|x| x.to_string()),
        ),
        ("--l1-gas", fee_args.l1_gas.map(|x| x.to_string())),
        (
            "--l1-gas-price",
            fee_args.l1_gas_price.map(|x| x.to_string()),
        ),
        ("--l2-gas", fee_args.l2_gas.map(|x| x.to_string())),
        (
            "--l2-gas-price",
            fee_args.l2_gas_price.map(|x| x.to_string()),
        ),
    ];

    for &(key, ref value) in &options {
        if let Some(val) = value.as_ref() {
            args.push(key);
            args.push(val);
        }
    }

    let snapbox = runner(&args).current_dir(tempdir.path());
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
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address[..]is not deployed[..]",
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
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        r#"Error: Function with selector "0x2e0f845a8d0319c5c37d558023299beec2a0155d415f41cca140a09e6877c67" not found in ABI of the contract"#,
    );
}

// TODO(#3116): Before, the error message included "Failed to deserialize param #2", but now it's an undecoded felt.
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
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: invoke
        Error: Transaction execution error [..]0x4661696c656420746f20646573657269616c697a6520706172616d202332[..]
        "},
    );
}

#[test]
fn test_too_low_gas() {
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
        "--l1-gas",
        "1",
        "--l1-gas-price",
        "1",
        "--l2-gas",
        "1",
        "--l2-gas-price",
        "1",
        "--l1-data-gas",
        "1",
        "--l1-data-gas-price",
        "1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: invoke
        Error: The transaction's resources don't cover validation or the minimal transaction fee
        "},
    );
}

#[tokio::test]
async fn test_happy_case_cairo_expression_calldata() {
    let tempdir = create_and_deploy_oz_account().await;

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--json",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nested_struct_fn",
        "--arguments",
        calldata,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
async fn test_happy_case_shell() {
    let tempdir = create_and_deploy_oz_account().await;
    let binary_path = cargo_bin!("sncast");
    let command = os_specific_shell(&Utf8PathBuf::from("tests/shell/invoke"));

    let snapbox = command
        .current_dir(tempdir.path())
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}
