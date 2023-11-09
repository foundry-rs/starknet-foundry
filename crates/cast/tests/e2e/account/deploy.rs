use crate::helpers::constants::{CONTRACTS_DIR, DEVNET_OZ_CLASS_HASH, URL};
use crate::helpers::fixtures::convert_to_hex;
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_address_from_keystore, get_transaction_hash,
    get_transaction_receipt, mint_token,
};
use camino::Utf8PathBuf;
use cast::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
use indoc::indoc;
use serde_json::Value;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::DeployAccount;
use std::{env, fs};
use tempfile::TempDir;
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let (created_dir, accounts_file) = create_account("3", false).await;

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let hash = get_transaction_hash(&out.stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(created_dir.join(accounts_file)).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
    assert_eq!(items["alpha-goerli"]["my_account"]["deployed"], true);

    fs::remove_dir_all(created_dir).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let (created_dir, accounts_file) = create_account("4", true).await;

    let args = vec![
        "--profile",
        "my_account",
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let hash = get_transaction_hash(&out.stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    fs::remove_dir_all(created_dir).unwrap();
}

#[test_case("{}", "error: No accounts defined for network alpha-goerli" ; "when empty file")]
#[test_case("{\"alpha-goerli\": {}}", "error: Account with name my_account does not exist" ; "when account name not present")]
#[test_case("{\"alpha-goerli\": {\"my_account\" : {}}}", "error: Couldn't get private key from accounts file" ; "when private key not present")]
#[test_case("{\"alpha-goerli\": {\"my_account\" : {\"private_key\": \"0x1\"}}}", "error: Couldn't get salt from accounts file" ; "when salt not present")]
fn test_account_deploy_error(accounts_content: &str, error: &str) {
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let accounts_file = "./accounts.json";
    fs::write(temp_dir.path().join(accounts_file), accounts_content).unwrap();

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "10000000000000000",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");
    assert!(stderr_str.contains(error));
}

#[tokio::test]
async fn test_too_low_max_fee() {
    let (created_dir, accounts_file) = create_account("5", false).await;

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "--wait",
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "1",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: account deploy
        error: Max fee is smaller than the minimal transaction cost (validation plus fee transfer)
    "#});

    fs::remove_dir_all(created_dir).unwrap();
}

#[tokio::test]
pub async fn test_invalid_class_hash() {
    let (created_dir, accounts_file) = create_account("9", true).await;

    let args = vec![
        "--profile",
        "my_account",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        "0x123",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: account deploy
        error: Provided class hash 0x123 does not exist
    "#});

    fs::remove_dir_all(created_dir).unwrap();
}

#[tokio::test]
pub async fn test_valid_class_hash() {
    let (created_dir, accounts_file) = create_account("10", true).await;

    let args = vec![
        "--profile",
        "my_account",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "10000000000000000",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
        command: account deploy
        transaction_hash: [..]
    "#});

    fs::remove_dir_all(created_dir).unwrap();
}

pub async fn create_account(salt: &str, add_profile: bool) -> (Utf8PathBuf, &str) {
    let created_dir = duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/constructor_with_params",
        "put",
        salt,
    );
    let accounts_file = "./accounts.json";
    let mut args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];
    if add_profile {
        args.push("--add-profile");
    }

    Command::new(cargo_bin!("sncast"))
        .current_dir(created_dir.path())
        .args(&args)
        .assert()
        .success();

    let contents = fs::read_to_string(created_dir.path().join(accounts_file)).unwrap();
    let items: Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at {path}");

    mint_token(
        items["alpha-goerli"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        9_999_999_999_999_999_999,
    )
    .await;
    let created_dir_utf8 =
        Utf8PathBuf::from_path_buf(created_dir.into_path()).expect("Path contains invalid UTF-8");
    (created_dir_utf8, accounts_file)
}

#[tokio::test]
pub async fn test_happy_case_keystore() {
    let keystore_path = "tests/data/keystore/my_key.json";
    let account_path = "tests/data/keystore/my_account_undeployed_happy_case_copy.json";

    fs::copy(
        "tests/data/keystore/my_account_undeployed_happy_case.json",
        account_path,
    )
    .unwrap();
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let address = get_address_from_keystore(keystore_path, account_path, KEYSTORE_PASSWORD_ENV_VAR);

    mint_token(
        &convert_to_hex(&address.to_string()),
        9_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(account_path).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
    assert_eq!(items["deployment"]["status"], "deployed");
    assert!(!items["deployment"]["address"].is_null());
    assert!(items["deployment"]["salt"].is_null());

    _ = fs::remove_file(account_path);
}

#[tokio::test]
pub async fn test_keystore_already_deployed() {
    let keystore_path = "tests/data/keystore/my_key.json";
    let account_path = "tests/data/keystore/account_copy.json";

    fs::copy("tests/data/keystore/my_account.json", account_path).unwrap();
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    snapbox.assert().stderr_matches(indoc! {r#"
        command: account deploy
        error: Account already deployed
    "#});

    _ = fs::remove_file(account_path);
}

#[tokio::test]
pub async fn test_keystore_key_mismatch() {
    let keystore_path = "tests/data/keystore/my_key_invalid.json";
    let account_path = "tests/data/keystore/my_account_copy.json";

    fs::copy(
        "tests/data/keystore/my_account_undeployed.json",
        account_path,
    )
    .unwrap();
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    snapbox.assert().stderr_matches(indoc! {r#"
        command: account deploy
        error: Public key and private key from keystore do not match
    "#});

    _ = fs::remove_file(account_path);
}

#[test_case("tests/data/keystore/my_key_inexistent.json", "tests/data/keystore/my_account_undeployed.json", "error: Couldn't read keystore file" ; "when inexistent keystore")]
#[test_case("tests/data/keystore/my_key.json", "tests/data/keystore/my_account_inexistent.json", "error: Couldn't read account file" ; "when inexistent account")]
pub fn test_deploy_keystore_inexistent_file(keystore_path: &str, account_path: &str, error: &str) {
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();
    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(stderr_str.contains(error));
}

#[tokio::test]
pub async fn test_deploy_keystore_no_status() {
    let keystore_path = "tests/data/keystore/my_key.json";
    let account_path = "tests/data/keystore/my_account_invalid.json";
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");
    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    snapbox.assert().stderr_matches(indoc! {r#"
        command: account deploy
        error: Failed to get status from account JSON file
    "#});
}

#[tokio::test]
pub async fn test_deploy_keystore_other_args() {
    let keystore_path = "tests/data/keystore/my_key.json";
    let account_path = "tests/data/keystore/my_account_undeployed_happy_case_other_args_copy.json";

    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    fs::copy(
        "tests/data/keystore/my_account_undeployed_happy_case_other_args.json",
        account_path,
    )
    .unwrap();

    let address = get_address_from_keystore(keystore_path, account_path, KEYSTORE_PASSWORD_ENV_VAR);

    mint_token(
        &convert_to_hex(&address.to_string()),
        9_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "accounts.json",
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--name",
        "some-name",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast")).args(args);
    snapbox.assert().stdout_matches(indoc! {r#"
        command: account deploy
        transaction_hash: 0x[..]
    "#});

    _ = fs::remove_file(account_path);
}
