use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::fixtures::{
    get_address_from_keystore, get_transaction_hash, get_transaction_receipt, load_json_file,
    load_native_accounts, mint_token,
};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use configuration::test_utils::copy_config_to_tempdir;
use conversions::string::IntoHexStr;
use indoc::indoc;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::{
    BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
};
use starknet_rust::core::types::TransactionReceipt::DeployAccount;
use std::fs;
use tempfile::{TempDir, tempdir};
use test_case::test_case;

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0, "oz"; "cairo_0_class_hash")]
#[test_case(&OZ_CLASS_HASH.into_hex_string(), "oz"; "cairo_1_class_hash")]
#[test_case(&READY_CLASS_HASH.into_hex_string(), "ready"; "ready_class_hash")]
#[test_case(&BRAAVOS_CLASS_HASH.into_hex_string(), "braavos"; "braavos_class_hash")]
#[tokio::test]
pub async fn test_happy_case(class_hash: &str, account_type: &str) {
    let tempdir = create_account(false, class_hash, account_type).await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let bdg = snapbox.assert();

    let hash = get_transaction_hash(&bdg.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = bdg.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let path = Utf8PathBuf::from_path_buf(tempdir.path().join(accounts_file))
        .expect("Path is not valid UTF-8");
    let document = load_json_file(&path).expect("Failed to load accounts");
    assert_eq!(document["version"], 2);
    assert_eq!(
        document["accounts"]["alpha-sepolia"]["my_account"]["signer"]["type"],
        "private_key"
    );
    let items = document["accounts"].clone();
    assert_eq!(items["alpha-sepolia"]["my_account"]["deployed"], true);
}

#[tokio::test]
pub async fn test_happy_case_max_fee() {
    let tempdir = create_account(false, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let bdg = snapbox.assert();

    let hash = get_transaction_hash(&bdg.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = bdg.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let path = Utf8PathBuf::from_path_buf(tempdir.path().join(accounts_file))
        .expect("Path is not valid UTF-8");
    let items = load_native_accounts(&path).expect("Failed to load accounts");
    assert_eq!(items["alpha-sepolia"]["my_account"]["deployed"], true);
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = create_account(true, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert();

    let hash = get_transaction_hash(&output.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = output.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));
}

#[test_case("{\"alpha-sepolia\": {}}", "Error: Account = my_account not found under network = alpha-sepolia" ; "when account name not present")]
#[test_case("{\"alpha-sepolia\": {\"my_account\" : {}}}", "Error: Failed to parse field `alpha-sepolia.my_account` in file 'accounts.json': missing field `public_key`[..]" ; "when public key not present")]
fn test_account_deploy_error(accounts_content: &str, error: &str) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let accounts_file = "accounts.json";
    fs::write(temp_dir.path().join(accounts_file), accounts_content).unwrap();

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert();

    assert_stderr_contains(output, error);
}

#[tokio::test]
pub async fn test_valid_class_hash() {
    let tempdir = create_account(true, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.voyager.online/tx/0x[..]
    "});
}

#[tokio::test]
pub async fn test_valid_no_max_fee() {
    let tempdir = create_account(true, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.voyager.online/tx/0x[..]
    "});
}

pub async fn create_account(add_profile: bool, class_hash: &str, account_type: &str) -> TempDir {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let accounts_file = "accounts.json";

    let mut args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--class-hash",
        class_hash,
        "--type",
        account_type,
    ];
    if add_profile {
        args.push("--add-profile");
        args.push("deploy_profile");
    }

    runner(&args).current_dir(tempdir.path()).assert().success();

    let path = Utf8PathBuf::from_path_buf(tempdir.path().join(accounts_file))
        .expect("Path is not valid UTF-8");
    let items = load_native_accounts(&path).expect("Failed to load accounts");

    mint_token(
        items["alpha-sepolia"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        9_999_999_999_999_999_999_999_999_999_999,
    )
    .await;
    tempdir
}

#[tokio::test]
pub async fn test_dry_run() {
    let tempdir = create_account(true, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
        "--dry-run",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Dry run completed

            Overall Fee: [..] Fri (~[..] STRK)
            "
        },
    );
}

#[tokio::test]
pub async fn test_dry_run_detailed() {
    let tempdir = create_account(true, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
        "--dry-run",
        "--detailed",
    ];
    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Dry run completed

            Overall Fee: [..] Fri (~[..] STRK)
            L1 Gas Consumed:      [..]
            L1 Gas Price:         [..]
            L2 Gas Consumed:      [..]
            L2 Gas Price:         [..]
            L1 Data Gas Consumed: [..]
            L1 Data Gas Price:    [..]
            "
        },
    );
}

#[tokio::test]
pub async fn test_json_output_format() {
    let tempdir = create_account(false, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    snapbox.assert().stdout_eq(indoc! {r#"
        {"command":"account deploy","transaction_hash":"0x0[..]","type":"response"}
        {"links":"transaction: https://sepolia.voyager.online/tx/0x0[..]","title":"account deployment","type":"notification"}
    "#});
}
