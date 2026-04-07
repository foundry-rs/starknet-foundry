use crate::e2e::ledger::{
    BRAAVOS_LEDGER_PATH, LEDGER_ACCOUNT_NAME, OZ_LEDGER_PATH, TEST_LEDGER_PATH, automation,
    create_temp_accounts_json, deploy_ledger_account, deploy_ledger_account_of_type,
    setup_speculos,
};
use crate::helpers::constants::{
    CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA, CONTRACTS_DIR,
    MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA, MULTICALL_CONFIGS_DIR, URL,
};
use crate::helpers::fixtures::{
    duplicate_contract_directory_with_salt, get_transaction_hash, get_transaction_receipt,
    join_tempdirs,
};
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::assert_stdout_contains;
use sncast::AccountType;
use starknet_rust::core::types::TransactionReceipt::{Declare, Invoke};
use starknet_types_core::felt::Felt;
use std::path::Path;
use tempfile::tempdir;
use test_case::test_case;

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_invoke_happy_case() {
    let (client, url) = setup_speculos(5001);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5001_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
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

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_invoke_with_wait() {
    let (client, url) = setup_speculos(5002);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5002_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    // Without `--ledger-path`, it should be taken from the accounts file
    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--json",
        "--wait",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x3 0x4",
    ];

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_deploy_happy_case() {
    let (client, url) = setup_speculos(5004);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5004_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--json",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x123",
        "--unique",
    ];

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_deploy_with_constructor() {
    let (client, url) = setup_speculos(5005);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(6001_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--json",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x456",
        "--unique",
        "--constructor-calldata",
        "0x1 0x1 0x0",
    ];

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_declare() {
    let (client, url) = setup_speculos(5006);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5006_u32)).await;

    let contract_dir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "ledger_declare",
    );
    let accounts_tempdir = create_temp_accounts_json(account_address);
    join_tempdirs(&accounts_tempdir, &contract_dir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .current_dir(contract_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Declare(_)));
}

#[test_case(AccountType::OpenZeppelin, "oz", Some(OZ_LEDGER_PATH), None, 5008; "oz_account_type")]
#[test_case(AccountType::Ready, "ready", None, Some(1), 5009; "ready_account_type")]
#[test_case(AccountType::Braavos, "braavos", Some(BRAAVOS_LEDGER_PATH), None, 5010; "braavos_account_type")]
#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_import_and_invoke(
    account_type: AccountType,
    account_type_str: &str,
    ledger_path: Option<&str>,
    ledger_account_id: Option<u32>,
    port: u16,
) {
    let (client, url) = setup_speculos(port);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
            automation::APPROVE_PUBLIC_KEY,
        ])
        .await
        .unwrap();

    let account_id_path_buf;
    let deploy_path = match (ledger_path, ledger_account_id) {
        (Some(path), None) => path,
        (None, Some(id)) => {
            account_id_path_buf = format!("m//starknet'/sncast'/0'/{id}'/0");
            &account_id_path_buf
        }
        _ => unreachable!(),
    };
    let account_address =
        deploy_ledger_account_of_type(&url, deploy_path, Felt::from(u32::from(port)), account_type)
            .await;
    let tempdir = tempdir().unwrap();
    let accounts_file = tempdir.path().join("accounts.json");
    let accounts_file_str = accounts_file.to_str().unwrap();
    let account_address_str = format!("{account_address:#x}");
    let ledger_account_id_str;
    let mut import_args = vec![
        "--accounts-file",
        accounts_file_str,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        LEDGER_ACCOUNT_NAME,
        "--address",
        &account_address_str,
        "--type",
        account_type_str,
    ];
    if let Some(path) = ledger_path {
        import_args.push("--ledger-path");
        import_args.push(path);
    } else if let Some(id) = ledger_account_id {
        ledger_account_id_str = id.to_string();
        import_args.push("--ledger-account-id");
        import_args.push(&ledger_account_id_str);
    }
    runner(&import_args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    let args = vec![
        "--accounts-file",
        accounts_file_str,
        "--account",
        LEDGER_ACCOUNT_NAME,
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

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_multicall() {
    let (client, url) = setup_speculos(5007);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5007_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let multicall_path = project_root::get_project_root().expect("failed to get project root path");
    let multicall_path = Path::new(&multicall_path)
        .join(MULTICALL_CONFIGS_DIR)
        .join("invoke_ledger.toml");
    let multicall_path = multicall_path
        .to_str()
        .expect("failed converting path to str");

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--json",
        "multicall",
        "run",
        "--url",
        URL,
        "--path",
        multicall_path,
    ];

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(tx_hash).await;
    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_invoke_dry_run() {
    let (_, url) = setup_speculos(5007);

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5007_u32)).await;
    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let output = runner(&[
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
        "--dry-run",
    ])
    .assert()
    .success();

    assert_stdout_contains(output, "Dry run completed");
}
