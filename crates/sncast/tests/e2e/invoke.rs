use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    create_and_deploy_account, create_and_deploy_oz_account, get_accounts_path,
    get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use snapbox::cmd::{Command, cargo_bin};
use sncast::AccountType;
use sncast::helpers::constants::{ARGENT_CLASS_HASH, OZ_CLASS_HASH};
use sncast::helpers::fee::FeeArgs;
use starknet::core::types::TransactionReceipt::Invoke;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::path::PathBuf;
use tempfile::tempdir;
use test_case::test_case;

#[tokio::test]
async fn test_happy_case_human_readable() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
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
            command: invoke
            transaction_hash: [..]

            To see invocation details, visit:
            transaction: [..]
            "
        },
    );
}

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
// TODO(#3089)
// #[test_case(BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case(class_hash: Felt, account_type: AccountType) {
    let tempdir = create_and_deploy_account(class_hash, account_type).await;
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
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

// TODO(#3116): Before, the error message included 'ENTRYPOINT_NOT_FOUND', but now it's an undecoded felt.
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
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {"
            command: invoke
            error: Transaction execution error [..]0x454e545259504f494e545f4e4f545f464f554e44[..]
        "},
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
        command: invoke
        error: Transaction execution error [..]0x4661696c656420746f20646573657269616c697a6520706172616d202332[..]
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
        command: invoke
        error: The transaction's resources don't cover validation or the minimal transaction fee
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
        "--int-format",
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

    let script_extension = if cfg!(windows) { ".ps1" } else { ".sh" };
    let test_path = PathBuf::from(format!("tests/shell/invoke{script_extension}"))
        .canonicalize()
        .unwrap();
    let binary_path = cargo_bin!("sncast");

    let command = if cfg!(windows) {
        Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(test_path)
    } else {
        Command::new(test_path)
    };

    let snapbox = command
        .current_dir(tempdir.path())
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}

// TODO(#3118): Remove this test, once integration with braavos is restored
#[tokio::test]
async fn test_braavos_disabled() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        &accounts_json_path,
        "--account",
        "braavos",
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

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Using Braavos accounts is temporarily disabled because they don't yet work with starknet 0.13.5.
            Visit this link to read more: https://community.starknet.io/t/starknet-devtools-for-0-13-5/115495#p-2359168-braavos-compatibility-issues-3
        "},
    );
}
