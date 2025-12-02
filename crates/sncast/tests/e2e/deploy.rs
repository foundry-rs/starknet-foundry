use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, CONTRACTS_DIR,
    DEVNET_OZ_CLASS_HASH_CAIRO_0, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    create_and_deploy_account, create_and_deploy_oz_account, create_test_provider,
    duplicate_contract_directory_with_salt, get_transaction_by_hash, get_transaction_hash,
    get_transaction_receipt, join_tempdirs,
};
use crate::helpers::runner::runner;
use crate::helpers::shell::os_specific_shell;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use snapbox::cmd::cargo_bin;
use sncast::AccountType;
use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::helpers::fee::FeeArgs;
use starknet_rust::core::types::TransactionReceipt::Invoke;
use starknet_rust::core::types::{
    BlockId, BlockTag, InvokeTransaction, Transaction, TransactionExecutionStatus,
};
use starknet_rust::providers::Provider;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use test_case::test_case;
use toml::Value;

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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Deployment completed

            Contract Address: 0x0[..]
            Transaction Hash: 0x0[..]

            To see deployment details, visit:
            contract: [..]
            transaction: [..]
            "
        },
    );
}

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(sncast::helpers::constants::READY_CLASS_HASH, AccountType::Ready; "READY_CLASS_HASH")]
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
    tip: None,
    estimate_tip: false,
}; "max_fee")]
#[test_case(FeeArgs{
    max_fee: None,
    l1_data_gas: Some(100_000),
    l1_data_gas_price: Some(10_000_000_000_000),
    l1_gas: Some(100_000),
    l1_gas_price: Some(10_000_000_000_000),
    l2_gas: Some(1_000_000_000),
    l2_gas_price: Some(100_000_000_000_000_000_000),
    tip: Some(100_000),
    estimate_tip: false,
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
        ("--tip", fee_args.tip.map(|x| x.to_string())),
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
    let Invoke(receipt) = get_transaction_receipt(hash).await else {
        panic!("Should be Invoke receipt");
    };
    assert_eq!(
        receipt.execution_result.status(),
        TransactionExecutionStatus::Succeeded
    );

    let Transaction::Invoke(InvokeTransaction::V3(tx)) = get_transaction_by_hash(hash).await else {
        panic!("Expected Invoke V3 transaction")
    };
    assert_eq!(tx.tip, fee_args.tip.unwrap_or(0));
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

    assert!(matches!(receipt, Invoke(_)));
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

    assert!(matches!(receipt, Invoke(_)));
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
        Command: deploy
        Error: Transaction execution error [..]Input too long for arguments[..]
        "},
    );
}

#[test]
fn test_class_hash_with_package() {
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
        "--package",
        "my_package",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--class-hash <CLASS_HASH>' cannot be used with '--package <PACKAGE>'
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
        Command: deploy
        Error: Transaction execution error [..] contract already deployed at address [..]
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
    // TODO Check extra blank line
    println!("====\n{}\n====", output.as_stderr());

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: deploy
        Error: The transaction's resources don't cover validation or the minimal transaction fee
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

#[tokio::test]
async fn test_json_output_format() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

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

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
            {"command":"deploy","contract_address":"0x[..]","transaction_hash":"0x[..]","type":"response"}
            {"links":"contract: https://sepolia.starkscan.co/contract/0x[..]\ntransaction: https://sepolia.starkscan.co/tx/0x[..]\n","title":"deployment","type":"notification"}
            "#},
    );
}

#[tokio::test]
async fn test_happy_case_with_declare() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "with_declare",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "deploy",
        "--url",
        URL,
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--contract-name",
        "Map",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Deployment completed

            Contract Address:         0x0[..]
            Class Hash:               0x0[..]
            Declare Transaction Hash: 0x0[..]
            Deploy Transaction Hash:  0x0[..]

            To see deployment details, visit:
            contract: [..]
            class: [..]
            deploy transaction: [..]
            declare transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_happy_case_with_already_declared() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "with_redeclare",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    {
        // Declare the contract first
        let args = vec![
            "--accounts-file",
            "accounts.json",
            "--account",
            "my_account",
            "declare",
            "--url",
            URL,
            "--contract-name",
            "Map",
        ];
        let args = apply_test_resource_bounds_flags(args);

        runner(&args).current_dir(tempdir.path()).assert().success();
    }

    // Deploy the contract with declaring
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "deploy",
        "--url",
        URL,
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--contract-name",
        "Map",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Deployment completed

            Contract Address: 0x0[..]
            Transaction Hash: 0x0[..]

            To see deployment details, visit:
            contract: [..]
            transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_happy_case_with_declare_nonce() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "with_declare_nonce",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let nonce = {
        // Get nonce
        let provider = create_test_provider();
        let args = vec![
            "--accounts-file",
            "accounts.json",
            "--json",
            "account",
            "list",
        ];

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.assert().success();

        let value: Value = serde_json::from_str(output.as_stdout()).unwrap();
        let account_address = value["my_account"]["address"].as_str().unwrap();

        provider
            .get_nonce(
                BlockId::Tag(BlockTag::Latest),
                Felt::from_hex(account_address).unwrap(),
            )
            .await
            .unwrap()
            .to_string()
    };

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "deploy",
        "--url",
        URL,
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--contract-name",
        "Map",
        "--nonce",
        nonce.as_str(),
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Deployment completed

            Contract Address:         0x0[..]
            Class Hash:               0x0[..]
            Declare Transaction Hash: 0x0[..]
            Deploy Transaction Hash:  0x0[..]

            To see deployment details, visit:
            contract: [..]
            class: [..]
            deploy transaction: [..]
            declare transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_deploy_with_declare_invalid_nonce() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "with_redeclare",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "deploy",
        "--url",
        URL,
        "--constructor-calldata",
        "0x1",
        "0x1",
        "0x0",
        "--contract-name",
        "Map",
        "--nonce",
        "0x123456",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {
            "
            Command: deploy
            Error: Invalid transaction nonce
            "
        },
    );
}
