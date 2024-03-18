use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH, URL};
use crate::helpers::fixtures::{convert_to_hex, copy_file};
use crate::helpers::fixtures::{
    get_address_from_keystore, get_transaction_hash, get_transaction_receipt, mint_token,
};
use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use indoc::indoc;
use serde_json::Value;
use shared::test_utils::output_assert::{assert_stderr_contains, AsOutput};
use sncast::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
use starknet::core::types::TransactionReceipt::DeployAccount;
use std::{env, fs};
use tempfile::{tempdir, TempDir};
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let tempdir = create_account(false).await;
    let accounts_file = "accounts.json";

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

    let snapbox = runner(&args).current_dir(tempdir.path());
    let bdg = snapbox.assert();

    let hash = get_transaction_hash(&bdg.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = bdg.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
    assert_eq!(items["alpha-goerli"]["my_account"]["deployed"], true);
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = create_account(true).await;
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
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let hash = get_transaction_hash(&output.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = output.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));
}

#[test_case("{}", "error: No accounts defined for network alpha-goerli" ; "when empty file")]
#[test_case("{\"alpha-goerli\": {}}", "error: Account with name my_account does not exist" ; "when account name not present")]
#[test_case("{\"alpha-goerli\": {\"my_account\" : {}}}", "error: Failed to get private key from accounts file" ; "when private key not present")]
#[test_case("{\"alpha-goerli\": {\"my_account\" : {\"private_key\": \"0x1\"}}}", "error: Failed to get salt from accounts file" ; "when salt not present")]
fn test_account_deploy_error(accounts_content: &str, error: &str) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let accounts_file = "accounts.json";
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

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert();

    assert_stderr_contains(output, error);
}

#[tokio::test]
async fn test_too_low_max_fee() {
    let tempdir = create_account(false).await;
    let accounts_file = "accounts.json";

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

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Max fee is smaller than the minimal transaction cost
        "},
    );
}

#[tokio::test]
pub async fn test_invalid_class_hash() {
    let tempdir = create_account(true).await;
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
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        "0x123",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Provided class hash 0x123 does not exist
        "},
    );
}

#[tokio::test]
pub async fn test_valid_class_hash() {
    let tempdir = create_account(true).await;
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
        "--max-fee",
        "10000000000000000",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account deploy
        transaction_hash: [..]
    "});
}

#[tokio::test]
pub async fn test_valid_no_max_fee() {
    let tempdir = create_account(true).await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--url",
        URL,
        "--profile",
        "deploy_profile",
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account deploy
        transaction_hash: [..]
    "});
}

pub async fn create_account(add_profile: bool) -> TempDir {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let accounts_file = "accounts.json";

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
        args.push("deploy_profile");
    }

    runner(&args).current_dir(tempdir.path()).assert().success();

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at {path}");

    mint_token(
        items["alpha-goerli"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        9_999_999_999_999_999_999,
    )
    .await;
    tempdir
}

#[tokio::test]
pub async fn test_happy_case_keystore() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_undeployed_happy_case.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        "tests/data/keystore/my_account_undeployed_happy_case.json",
        tempdir.path().join(account_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let address = get_address_from_keystore(
        tempdir.path().join(keystore_file).to_str().unwrap(),
        tempdir.path().join(account_file).to_str().unwrap(),
        KEYSTORE_PASSWORD_ENV_VAR,
    );

    mint_token(
        &convert_to_hex(&address.to_string()),
        9_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account deploy
        transaction_hash: 0x[..]
    "});

    let contents = fs::read_to_string(tempdir.path().join(account_file)).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
    assert_eq!(items["deployment"]["status"], "deployed");
    assert!(!items["deployment"]["address"].is_null());
    assert!(items["deployment"]["salt"].is_null());
}

#[tokio::test]
pub async fn test_keystore_already_deployed() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "account.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        "tests/data/keystore/my_account.json",
        tempdir.path().join(account_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Account already deployed
        "},
    );
}

#[tokio::test]
pub async fn test_keystore_key_mismatch() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key_invalid.json";
    let account_file = "my_account_undeployed.json";

    copy_file(
        "tests/data/keystore/my_key_invalid.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        "tests/data/keystore/my_account_undeployed.json",
        tempdir.path().join(account_file),
    );

    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Public key and private key from keystore do not match
        "},
    );
}

#[tokio::test]
pub async fn test_deploy_keystore_inexistent_keystore_file() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key_inexistent.json";
    let account_file = "my_account_undeployed.json";

    copy_file(
        "tests/data/keystore/my_account_undeployed.json",
        tempdir.path().join(account_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Failed to read keystore file
        "},
    );
}

#[tokio::test]
pub async fn test_deploy_keystore_inexistent_account_file() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_inexistent.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Failed to read account file[..]
        "},
    );
}

#[tokio::test]
pub async fn test_deploy_keystore_no_status() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_invalid.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        "tests/data/keystore/my_account_invalid.json",
        tempdir.path().join(account_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--max-fee",
        "10000000000000000",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account deploy
        error: Failed to get status from account JSON file
        "},
    );
}

#[tokio::test]
pub async fn test_deploy_keystore_other_args() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_undeployed_happy_case_other_args.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        "tests/data/keystore/my_account_undeployed_happy_case_other_args.json",
        tempdir.path().join(account_file),
    );
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let address = get_address_from_keystore(
        tempdir.path().join(keystore_file),
        tempdir.path().join(account_file),
        KEYSTORE_PASSWORD_ENV_VAR,
    );

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
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--name",
        "some-name",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().stdout_matches(indoc! {r"
        command: account deploy
        transaction_hash: 0x[..]
    "});
}
