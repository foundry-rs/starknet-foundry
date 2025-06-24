use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
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
use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::helpers::fee::FeeArgs;
use starknet::core::types::TransactionReceipt::Deploy;
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
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
#[test_case(sncast::helpers::constants::ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
#[test_case(sncast::helpers::constants::BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case(class_hash: Felt, account_type: AccountType) {
    let tempdir = create_and_deploy_account(class_hash, account_type).await;
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--json",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());

    let hash = get_transaction_hash(&snapbox.assert().success().get_output().stdout.clone());
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
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
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x2",
        "--unique",
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

    assert!(matches!(receipt, Deploy(_)));
}

#[tokio::test]
async fn test_happy_case_with_constructor() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user4",
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
    ];
    let args = apply_test_resource_bounds_flags(args);

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
        "--json",
        "deploy",
        "--url",
        URL,
        "--arguments",
        "0x420, 0x2137_u256",
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Deploy(_)));
}

// TODO(#3116): Before, this test returned message 'Input too long for arguments').
// Now, it returns message about transaction execution error.
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
        error: Transaction execution error [..]
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

// TODO(#3116): Before, this test returned message containing info that contract is already deployed.
// Now, it returns message about transaction execution error.
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
        error: Transaction execution error [..]
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
    let binary_path = cargo_bin!("sncast");
    let command = os_specific_shell(&Utf8PathBuf::from("tests/shell/deploy"));

    let snapbox = command
        .current_dir(tempdir.path())
        .arg(binary_path)
        .arg(URL)
        .arg(CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA);
    snapbox.assert().success();
}
