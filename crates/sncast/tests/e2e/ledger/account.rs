use crate::e2e::ledger::{
    LEDGER_ACCOUNT_NAME, LEDGER_PUBLIC_KEY, TEST_LEDGER_PATH, automation,
    create_undeployed_accounts_json, setup_speculos,
};
use crate::helpers::constants::URL;
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::mint_token;
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use starknet_types_core::felt::Felt;
use tempfile::tempdir;

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_create_ledger_account() {
    let (client, url) = setup_speculos(6001);
    let tempdir = tempdir().unwrap();

    // account create with ledger asks for public key confirmation on device
    client
        .automation(&[automation::APPROVE_PUBLIC_KEY])
        .await
        .unwrap();

    let output = runner(&[
        "--accounts-file",
        "accounts.json",
        "--ledger-path",
        TEST_LEDGER_PATH,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        LEDGER_ACCOUNT_NAME,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .current_dir(tempdir.path())
    .assert()
    .success();

    assert_stdout_contains(output, "Address: 0x[..]");

    // Verify accounts.json was written with the ledger public key
    let accounts_content = std::fs::read_to_string(tempdir.path().join("accounts.json")).unwrap();
    assert!(
        accounts_content.contains(LEDGER_PUBLIC_KEY),
        "accounts.json should contain ledger public key"
    );
    assert!(
        accounts_content.contains(TEST_LEDGER_PATH),
        "accounts.json should contain ledger derivation path"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_deploy_ledger_account() {
    let (client, url) = setup_speculos(6002);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let salt = Felt::from(7001_u32);
    let (address, tempdir) = create_undeployed_accounts_json(salt);
    let accounts_file = tempdir.path().join("accounts.json");

    mint_token(&format!("{address:#066x}"), u128::MAX).await;

    let args = apply_test_resource_bounds_flags(vec![
        "--accounts-file",
        accounts_file.to_str().unwrap(),
        "--account",
        LEDGER_ACCOUNT_NAME,
        "--ledger-path",
        TEST_LEDGER_PATH,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
    ]);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    assert_stdout_contains(output, r#"[..]"transaction_hash"[..]"#);
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_invalid_derivation_path() {
    let (_client, url) = setup_speculos(5001);

    let output = runner(&[
        "ledger",
        "get-public-key",
        "--path",
        "invalid/path",
        "--no-display",
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .assert()
    .failure();

    assert_stderr_contains(
        output,
        "error: invalid Ledger derivation path: EIP-2645 paths must have 6 levels",
    );
}
