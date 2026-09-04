use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::fixtures::get_accounts_path;
use crate::helpers::fixtures::{get_transaction_hash, get_transaction_receipt, mint_token};
use crate::helpers::insta::set_snapshot_suffix;
use crate::helpers::runner::runner;
use camino::Utf8Path;
use configuration::test_utils::copy_config_to_tempdir;
use conversions::string::IntoHexStr;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use sncast::helpers::constants::{BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH};
use starknet_rust::core::types::TransactionReceipt::DeployAccount;
use std::fs;
use tempfile::{TempDir, tempdir};
use test_case::test_case;

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0, "oz", "oz_cairo_0_class_hash"; "oz_cairo_0_class_hash")]
#[test_case(&OZ_CLASS_HASH.into_hex_string(), "oz", "oz_cairo_0_class_hash"; "oz_cairo_1_class_hash")]
#[test_case(&READY_CLASS_HASH.into_hex_string(), "ready", "ready_class_hash"; "ready_class_hash")]
#[test_case(&BRAAVOS_CLASS_HASH.into_hex_string(), "braavos", "braaavos_class_hash"; "braavos_class_hash")]
#[tokio::test]
pub async fn test_happy_case(class_hash: &str, account_type: &str, case_name: &str) {
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
        .current_dir(tempdir.path())
        .assert()
        .success();

    let hash = get_transaction_hash(&snapbox.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    snapbox.stdout_eq(indoc!(
        r#"
        {"command":"account deploy","transaction_hash":"[..]","type":"response"}
        {"links":"transaction: [..]","title":"account deployment","type":"notification"}
    "#
    ));

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("{case_name}");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
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
        .current_dir(tempdir.path())
        .assert()
        .success();

    let hash = get_transaction_hash(&snapbox.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    snapbox.stdout_eq(indoc!(
        r#"
        {"command":"account deploy","transaction_hash":"[..]","type":"response"}
        {"links":"transaction: [..]","title":"account deployment","type":"notification"}
    "#
    ));

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
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
        .current_dir(tempdir.path())
        .assert()
        .success();

    let hash = get_transaction_hash(&snapbox.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    snapbox.stdout_eq(indoc!(
        r#"
        {"command":"account deploy","transaction_hash":"[..]","type":"response"}
        {"links":"transaction: [..]","title":"account deployment","type":"notification"}
    "#
    ));

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("accounts.json");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });

    let snfoundry_toml_contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read accounts.json");

    set_snapshot_suffix!("snfoundry.toml");
    insta::assert_snapshot!(snfoundry_toml_contents);
}

#[tokio::test]
async fn test_account_deploy_error_non_existent_account() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = temp_dir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "non_existent_account",
    ];

    let snapbox = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .failure();

    snapbox.stderr_eq(indoc!(
        "
        Command: account deploy
        Error: Account = non_existent_account not found under network = alpha-sepolia
        "
    ));
}

#[tokio::test]
async fn test_account_deploy_error_when_public_key_not_present() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts_without_public_key.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = temp_dir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

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

    let snapbox = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .failure();

    snapbox.stderr_eq(indoc!("
        Command: account deploy
        Error: invalid schema of field accounts.alpha-sepolia.my_account in the accounts file: missing field `public_key` at line 7 column 7

        Caused by:
            missing field `public_key` at line 7 column 7
    "));
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

    let accounts_json_contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read accounts.json");
    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Unable to parse accounts.json");

    let account_address = accounts_json["accounts"]["alpha-sepolia"]["my_account"]["address"]
        .as_str()
        .expect("Unable to get the address of my_account");

    mint_token(account_address, 9_999_999_999_999_999_999_999_999_999_999).await;
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
