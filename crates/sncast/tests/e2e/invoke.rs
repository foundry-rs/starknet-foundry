use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fixtures::{
    create_and_deploy_account, create_and_deploy_oz_account, get_transaction_hash,
    get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use snapbox::cmd::{Command, cargo_bin};
use sncast::AccountType;
use sncast::helpers::constants::{ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH};
use starknet::core::types::TransactionReceipt::Invoke;
use starknet_types_core::felt::Felt;
use std::path::PathBuf;
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
        "--l1-gas",
        "100000",
        "--l1-gas-price",
        "10000000000000",
        "--l2-gas",
        "1000000000",
        "--l2-gas-price",
        "100000000000000000000",
        "--l1-data-gas",
        "100000",
        "--l1-data-gas-price",
        "10000000000000",
    ];

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
// #[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
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
        "--l1-gas",
        "100000",
        "--l1-gas-price",
        "10000000000000",
        "--l2-gas",
        "1000000000",
        "--l2-gas-price",
        "100000000000000000000",
        "--l1-data-gas",
        "100000",
        "--l1-data-gas-price",
        "10000000000000",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let hash = get_transaction_hash(&stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

// FIXME
// #[test_case(Some("99999999999999999"), None, None; "max_fee")]
// #[test_case(None, Some("999"), None; "max_gas")]
// #[test_case(None, None, Some("999999999999"); "max_gas_unit_price")]
// #[test_case(None, None, None; "none")]
// #[test_case(Some("99999999999999999"), None, Some("999999999999"); "max_fee_max_gas_unit_price")]
// #[test_case(None, Some("999"), Some("999999999999"); "max_gas_max_gas_unit_price")]
// #[test_case(Some("999999999999999"), Some("999"), None; "max_fee_max_gas")]
// #[tokio::test]
// async fn test_happy_case_different_fees(
//     max_fee: Option<&str>,
//     max_gas: Option<&str>,
//     max_gas_unit_price: Option<&str>,
// ) {
//     let tempdir = create_and_deploy_oz_account().await;
//     let mut args = vec![
//         "--accounts-file",
//         "accounts.json",
//         "--account",
//         "my_account",
//         "--int-format",
//         "--json",
//         "invoke",
//         "--url",
//         URL,
//         "--contract-address",
//         MAP_CONTRACT_ADDRESS_SEPOLIA,
//         "--function",
//         "put",
//         "--calldata",
//         "0x1 0x2",
//     ];
//     let options = [
//         ("--max-fee", max_fee),
//         ("--max-gas", max_gas),
//         ("--max-gas-unit-price", max_gas_unit_price),
//     ];

//     for &(key, value) in &options {
//         if let Some(val) = value {
//             args.append(&mut vec![key, val]);
//         }
//     }

//     let snapbox = runner(&args).current_dir(tempdir.path());
//     let output = snapbox.assert().success().get_output().stdout.clone();

//     let hash = get_transaction_hash(&output);
//     let receipt = get_transaction_receipt(hash).await;

//     assert!(matches!(receipt, Invoke(_)));
// }

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
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {"
            command: invoke
            error: Transaction execution error[..]('ENTRYPOINT_NOT_FOUND')[..]
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
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: invoke
        error: [..]Failed to deserialize param #2[..]
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

// #[test]
// fn test_calculated_max_gas_equal_to_zero_when_max_fee_passed() {
//     let args = vec![
//         "--accounts-file",
//         ACCOUNT_FILE_PATH,
//         "--account",
//         "user11",
//         "--wait",
//         "invoke",
//         "--url",
//         URL,
//         "--contract-address",
//         MAP_CONTRACT_ADDRESS_SEPOLIA,
//         "--function",
//         "put",
//         "--calldata",
//         "0x1",
//         "0x2",
//         "--max-fee",
//         "999999",
//     ];

//     let snapbox = runner(&args);
//     let output = snapbox.assert().success();

//     // TODO(#2852)
//     assert_stderr_contains(
//         output,
//         indoc! {r"
//         command: invoke
//         error: Calculated max-gas from provided --max-fee and the current network gas price is 0. Please increase --max-fee to obtain a positive gas amount: Tried to create NonZeroFelt from 0
//         "},
//     );
// }

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
        "--l1-gas",
        "100000",
        "--l1-gas-price",
        "10000000000000",
        "--l2-gas",
        "1000000000",
        "--l2-gas-price",
        "100000000000000000000",
        "--l1-data-gas",
        "100000",
        "--l1-data-gas-price",
        "10000000000000",
    ];

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
