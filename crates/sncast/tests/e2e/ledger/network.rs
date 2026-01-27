use crate::e2e::ledger::{
    LEDGER_ACCOUNT_NAME, TEST_LEDGER_PATH, automation, create_temp_accounts_json,
    deploy_ledger_account, setup_speculos,
};
use crate::helpers::constants::{
    CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    duplicate_contract_directory_with_salt, get_transaction_hash, join_tempdirs,
};
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::assert_stdout_contains;
use starknet_types_core::felt::Felt;

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
        "--ledger-path",
        TEST_LEDGER_PATH,
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

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    assert_ne!(tx_hash, Felt::ZERO, "Transaction hash should not be zero");
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

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--ledger-path",
        TEST_LEDGER_PATH,
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
    let args = apply_test_resource_bounds_flags(args);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    assert_stdout_contains(output, r#"[..]"transaction_hash"[..]"#);
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_multiple_invokes() {
    let (client, url) = setup_speculos(5003);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(6002_u32)).await;

    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");
    let accounts_file_str = accounts_file.to_str().unwrap();

    let base_args = vec![
        "--accounts-file",
        accounts_file_str,
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--ledger-path",
        TEST_LEDGER_PATH,
        "--json",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
    ];

    let args1 = apply_test_resource_bounds_flags({
        let mut a = base_args.clone();
        a.push("--calldata");
        a.push("0x10 0x20");
        a
    });
    let output1 = runner(&args1)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    let args2 = apply_test_resource_bounds_flags({
        let mut a = base_args.clone();
        a.push("--calldata");
        a.push("0x30 0x40");
        a
    });
    let output2 = runner(&args2)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    let hash1 = get_transaction_hash(&output1.get_output().stdout.clone());
    let hash2 = get_transaction_hash(&output2.get_output().stdout.clone());
    assert_ne!(hash1, hash2, "Transaction hashes should be different");
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

    let account_address = deploy_ledger_account(&url, TEST_LEDGER_PATH, Felt::from(5003_u32)).await;

    let tempdir = create_temp_accounts_json(account_address);
    let accounts_file = tempdir.path().join("accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--ledger-path",
        TEST_LEDGER_PATH,
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
    let args = apply_test_resource_bounds_flags(args);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    assert_ne!(tx_hash, Felt::ZERO, "Transaction hash should not be zero");
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
        "--ledger-path",
        TEST_LEDGER_PATH,
        "--json",
        "--wait",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--salt",
        "0x456",
        "--unique",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    assert_stdout_contains(output, r#"[..]"transaction_hash"[..]"#);
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
        "--ledger-path",
        TEST_LEDGER_PATH,
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .current_dir(contract_dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let tx_hash = get_transaction_hash(&output);
    assert_ne!(tx_hash, Felt::ZERO, "Transaction hash should not be zero");
}
