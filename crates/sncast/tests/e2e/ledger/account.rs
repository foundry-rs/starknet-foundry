use std::fs;

use crate::e2e::ledger::{
    BRAAVOS_LEDGER_PATH, LEDGER_ACCOUNT_NAME, LEDGER_PUBLIC_KEY, OZ_LEDGER_PATH, READY_LEDGER_PATH,
    TEST_LEDGER_PATH, automation, setup_speculos,
};
use crate::helpers::constants::URL;
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{get_transaction_hash, get_transaction_receipt, mint_token};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use configuration::test_utils::copy_config_to_tempdir;
use conversions::string::IntoHexStr;
use indoc::indoc;
use serde_json::{json, to_string_pretty};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use snapbox::assert_data_eq;
use sncast::helpers::account::load_accounts;
use sncast::helpers::constants::{BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH};
use speculos_client::AutomationRule;
use starknet_rust::core::types::TransactionReceipt::DeployAccount;
use tempfile::tempdir;
use test_case::test_case;

#[test_case("oz", "open_zeppelin", OZ_CLASS_HASH.into_hex_string(), 6001, &[automation::APPROVE_PUBLIC_KEY]; "oz_account_type")]
#[test_case("ready", "ready", READY_CLASS_HASH.into_hex_string(), 6004, &[automation::APPROVE_PUBLIC_KEY]; "ready_account_type")]
// Braavos calls sign_hash twice during fee estimation (tx_hash + aux_hash) because
// is_signer_interactive() always returns false — see BraavosAccountFactory::is_signer_interactive.
// That means we need ENABLE_BLIND_SIGN + two APPROVE_BLIND_SIGN_HASH after the public key approval.
#[test_case(
    "braavos", "braavos", BRAAVOS_CLASS_HASH.into_hex_string(), 6007,
    &[
        automation::APPROVE_PUBLIC_KEY,
        automation::ENABLE_BLIND_SIGN,
        automation::APPROVE_BLIND_SIGN_HASH, // tx_hash
        automation::APPROVE_BLIND_SIGN_HASH, // aux_hash
    ];
    "braavos_account_type"
)]
#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_create_ledger_account(
    account_type: &str,
    saved_type: &str,
    class_hash: String,
    port: u16,
    automations: &[speculos_client::AutomationRule<'static>],
) {
    let (client, url) = setup_speculos(port);
    let tempdir = tempdir().unwrap();

    client.automation(automations).await.unwrap();

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
        "--type",
        account_type,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
    .current_dir(tempdir.path())
    .assert()
    .success();

    assert_stdout_contains(
        output,
        format!(
            indoc! {"
                Please confirm the public key on your Ledger device...
                Connected to Ledger device

                Success: Account created

                Address: 0x0[..]

                Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

                After prefunding the account, run:
                sncast --accounts-file accounts.json --ledger-path \"{TEST_LEDGER_PATH}\" account deploy --url {URL} --name {LEDGER_ACCOUNT_NAME}

                To see account creation details, visit:
                account: [..]
            "},
            TEST_LEDGER_PATH = TEST_LEDGER_PATH,
            URL = URL,
            LEDGER_ACCOUNT_NAME = LEDGER_ACCOUNT_NAME,
        ),
    );

    let contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read created file");

    let expected = json!(
        {
            "alpha-sepolia": {
                LEDGER_ACCOUNT_NAME: {
                    "address": "0x[..]",
                    "class_hash": class_hash,
                    "deployed": false,
                    "legacy": false,
                    "public_key": "0x[..]",
                    "salt": "0x[..]",
                    "type": saved_type,
                    "ledger_path": TEST_LEDGER_PATH,
                }
            }
        }
    );

    assert_data_eq!(contents, to_string_pretty(&expected).unwrap());
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_create_ledger_account_add_profile() {
    let (client, url) = setup_speculos(6008);
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);

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
        "--add-profile",
        LEDGER_ACCOUNT_NAME,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .current_dir(tempdir.path())
    .assert()
    .success();

    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile: Profile {LEDGER_ACCOUNT_NAME} successfully added to {config_path}"),
    );

    let contents = std::fs::read_to_string(tempdir.path().join("snfoundry.toml")).unwrap();
    assert!(contents.contains(&format!("[sncast.{LEDGER_ACCOUNT_NAME}]")));
    assert!(contents.contains(&format!("account = \"{LEDGER_ACCOUNT_NAME}\"")));
}

#[test_case(
    "oz", OZ_LEDGER_PATH, 6017,
    // create: public key only (OZ skips signing during fee estimation)
    // deploy: enable blind sign + 1 sign_hash
    &[
        automation::APPROVE_PUBLIC_KEY,
        automation::ENABLE_BLIND_SIGN,
        automation::APPROVE_BLIND_SIGN_HASH,
    ];
    "oz_account_type"
)]
#[test_case(
    "ready", READY_LEDGER_PATH, 6018,
    // create: public key only (Ready skips signing during fee estimation)
    // deploy: enable blind sign + 1 sign_hash
    &[
        automation::APPROVE_PUBLIC_KEY,
        automation::ENABLE_BLIND_SIGN,
        automation::APPROVE_BLIND_SIGN_HASH,
    ];
    "ready_account_type"
)]
#[test_case(
    "braavos", BRAAVOS_LEDGER_PATH, 6019,
    // create: public key + enable blind sign + 2x sign_hash (tx_hash + aux_hash)
    // deploy: 2x sign_hash again (tx_hash + aux_hash), blind sign already enabled
    &[
        automation::APPROVE_PUBLIC_KEY,
        automation::ENABLE_BLIND_SIGN,
        automation::APPROVE_BLIND_SIGN_HASH, // create: tx_hash
        automation::APPROVE_BLIND_SIGN_HASH, // create: aux_hash
        automation::APPROVE_BLIND_SIGN_HASH, // deploy: tx_hash
        automation::APPROVE_BLIND_SIGN_HASH, // deploy: aux_hash
    ];
    "braavos_account_type"
)]
#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_deploy_ledger_account(
    account_type_str: &str,
    ledger_path: &str,
    port: u16,
    automations: &[AutomationRule<'static>],
) {
    let (client, url) = setup_speculos(port);

    client.automation(automations).await.unwrap();

    let tempdir = tempdir().unwrap();
    let accounts_file = tempdir.path().join("accounts.json");
    let accounts_file_str = accounts_file.to_str().unwrap();

    // First create the accounts
    runner(&[
        "--accounts-file",
        accounts_file_str,
        "--ledger-path",
        ledger_path,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        LEDGER_ACCOUNT_NAME,
        "--type",
        account_type_str,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .assert()
    .success();

    let accounts_content = std::fs::read_to_string(&accounts_file).unwrap();
    let contents_json: serde_json::Value = serde_json::from_str(&accounts_content).unwrap();
    let address = contents_json["alpha-sepolia"][LEDGER_ACCOUNT_NAME]["address"]
        .as_str()
        .unwrap()
        .to_string();

    mint_token(&address, u128::MAX).await;

    let args = apply_test_resource_bounds_flags(vec![
        "--accounts-file",
        accounts_file_str,
        "--ledger-path",
        ledger_path,
        "--json",
        "account",
        "deploy",
        "--name",
        LEDGER_ACCOUNT_NAME,
        "--url",
        URL,
    ]);

    let output = runner(&args)
        .env("LEDGER_EMULATOR_URL", &url)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .assert()
        .success();

    let hash = get_transaction_hash(&output.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = output.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let path = Utf8PathBuf::from_path_buf(accounts_file.clone()).expect("Path is not valid UTF-8");
    let items = load_accounts(&path).expect("Failed to load accounts");
    assert_eq!(
        items["alpha-sepolia"][LEDGER_ACCOUNT_NAME]["deployed"],
        true
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_invalid_derivation_path() {
    let (_client, url) = setup_speculos(6011);

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

#[test_case("oz", "open_zeppelin", OZ_CLASS_HASH.into_hex_string(), 6003; "oz_account_type")]
#[test_case("ready", "ready", READY_CLASS_HASH.into_hex_string(), 6006; "ready_account_type")]
#[test_case("braavos", "braavos", BRAAVOS_CLASS_HASH.into_hex_string(), 6007; "braavos_account_type")]
#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_import_ledger_account(
    account_type: &str,
    saved_type: &str,
    class_hash: String,
    port: u16,
) {
    let (client, url) = setup_speculos(port);
    let tempdir = tempdir().unwrap();

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
        "import",
        "--url",
        URL,
        "--name",
        LEDGER_ACCOUNT_NAME,
        "--address",
        "0x1",
        "--class-hash",
        &class_hash,
        "--type",
        account_type,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .current_dir(tempdir.path())
    .assert()
    .success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: Account imported successfully

            Account Name: my_ledger
        "},
    );

    let accounts_content = std::fs::read_to_string(tempdir.path().join("accounts.json")).unwrap();
    let contents_json: serde_json::Value = serde_json::from_str(&accounts_content).unwrap();
    assert_eq!(
        contents_json,
        json!({
            "alpha-sepolia": {
                LEDGER_ACCOUNT_NAME: {
                    "address": "0x1",
                    "class_hash": class_hash,
                    "deployed": false,
                    "legacy": false,
                    "public_key": LEDGER_PUBLIC_KEY,
                    "type": saved_type,
                    "ledger_path": TEST_LEDGER_PATH,
                }
            }
        })
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_import_ledger_account_add_profile() {
    let (client, url) = setup_speculos(6010);
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);

    client
        .automation(&[automation::APPROVE_PUBLIC_KEY])
        .await
        .unwrap();

    let oz_class_hash = OZ_CLASS_HASH.into_hex_string();

    let output = runner(&[
        "--accounts-file",
        "accounts.json",
        "--ledger-path",
        TEST_LEDGER_PATH,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        LEDGER_ACCOUNT_NAME,
        "--address",
        "0x1",
        "--class-hash",
        &oz_class_hash,
        "--type",
        "oz",
        "--add-profile",
        LEDGER_ACCOUNT_NAME,
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .current_dir(tempdir.path())
    .assert()
    .success();

    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile:  Profile {LEDGER_ACCOUNT_NAME} successfully added to {config_path}"),
    );

    let contents = std::fs::read_to_string(tempdir.path().join("snfoundry.toml")).unwrap();
    assert!(contents.contains(&format!("[sncast.{LEDGER_ACCOUNT_NAME}]")));
    assert!(contents.contains(&format!("account = \"{LEDGER_ACCOUNT_NAME}\"")));
}
