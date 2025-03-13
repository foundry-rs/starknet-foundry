use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_CLASS_HASH_SEPOLIA, TEST_RESOURCE_BOUNDS_FLAGS, URL,
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
use sncast::helpers::constants::OZ_CLASS_HASH;
use starknet::core::types::TransactionReceipt::Deploy;
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
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
    ]
    .into_iter()
    .chain(TEST_RESOURCE_BOUNDS_FLAGS.into_iter())
    .collect::<Vec<&str>>();

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            command: deploy
            contract_address: 0x0[..]
            transaction_hash: 0x0[..]

            To see deployment details, visit:
            contract: [..]
            transaction: [..]
            "
        },
    );
}

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
// TODO(#3089)
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
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
    ]
    .into_iter()
    .chain(TEST_RESOURCE_BOUNDS_FLAGS.into_iter())
    .collect::<Vec<&str>>();

    let snapbox = runner(&args).current_dir(tempdir.path());

    let hash = get_transaction_hash(&snapbox.assert().success().get_output().stdout.clone());
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

// TODO(#3090)
// #[test_case(Some("99999999999999999"), None, None; "max_fee")]
// #[test_case(None, Some("999"), None; "max_gas")]
// #[test_case(None, None, Some("999999999999"); "max_gas_unit_price")]
// #[test_case(None, None, None; "none")]
// #[test_case(Some("999999999999999"), None, Some("999999999999"); "max_fee_max_gas_unit_price")]
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
//         "deploy",
//         "--url",
//         URL,
//         "--class-hash",
//         MAP_CONTRACT_CLASS_HASH_SEPOLIA,
//         "--salt",
//         "0x2",
//         "--unique",
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

//     assert!(matches!(receipt, Deploy(_)));
// }

#[tokio::test]
async fn test_happy_case_with_constructor() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user4",
        "--int-format",
        "--json",
        "deploy",
        "--url",
        URL,
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    ]
    .into_iter()
    .chain(TEST_RESOURCE_BOUNDS_FLAGS.into_iter())
    .collect::<Vec<&str>>();

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[tokio::test]
async fn test_happy_case_with_constructor_cairo_expression_calldata() {
    let tempdir = create_and_deploy_oz_account().await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "deploy",
        "--url",
        URL,
        "--arguments",
        "0x420, 0x2137_u256",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    ]
    .into_iter()
    .chain(TEST_RESOURCE_BOUNDS_FLAGS.into_iter())
    .collect::<Vec<&str>>();

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

#[test]
fn test_wrong_calldata() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user9",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--constructor-calldata",
        "0x1 0x2 0x3 0x4",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: [..]('Input too long for arguments')[..]
        "},
    );
}

#[tokio::test]
async fn test_contract_not_declared() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        ACCOUNT,
        "deploy",
        "--url",
        URL,
        "--class-hash",
        "0x1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Class with hash[..]is not declared[..]",
    );
}

#[test]
fn test_contract_already_deployed() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user1",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: [..]Deployment failed: contract already deployed at address [..]
        "},
    );
}

#[test]
fn test_too_low_gas() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user7",
        "--wait",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
        "--l1-gas",
        "1",
        "--l2-gas",
        "1",
        "--l1-data-gas",
        "1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: deploy
        error: The transaction's resources don't cover validation or the minimal transaction fee
        "},
    );
}

#[tokio::test]
async fn test_happy_case_shell() {
    let tempdir = create_and_deploy_oz_account().await;

    let script_extension = if cfg!(windows) { ".ps1" } else { ".sh" };
    let test_path = PathBuf::from(format!("tests/shell/deploy{script_extension}"))
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
        .arg(CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA);
    snapbox.assert().success();
}
