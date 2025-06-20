use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::env::set_keystore_password_env;
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::copy_file;
use crate::helpers::fixtures::{
    get_address_from_keystore, get_transaction_hash, get_transaction_receipt, mint_token,
};
use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use conversions::string::IntoHexStr;
use indoc::indoc;
use serde_json::Value;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use sncast::AccountType;
use sncast::helpers::constants::{
    ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, KEYSTORE_PASSWORD_ENV_VAR, OZ_CLASS_HASH,
};
use starknet::core::types::TransactionReceipt::DeployAccount;
use std::fs;
use tempfile::{TempDir, tempdir};
use test_case::test_case;

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0, "oz"; "cairo_0_class_hash")]
#[test_case(&OZ_CLASS_HASH.into_hex_string(), "oz"; "cairo_1_class_hash")]
#[test_case(&ARGENT_CLASS_HASH.into_hex_string(), "argent"; "argent_class_hash")]
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
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let bdg = snapbox.assert();

    let hash = get_transaction_hash(&bdg.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = bdg.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: Value = serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
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
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let bdg = snapbox.assert();

    let hash = get_transaction_hash(&bdg.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = bdg.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: Value = serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
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
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    let hash = get_transaction_hash(&output.get_output().stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str = output.as_stdout();
    assert!(stdout_str.contains("account deploy"));
    assert!(stdout_str.contains("transaction_hash"));
}

#[test_case("{\"alpha-sepolia\": {}}", "Error: Account = my_account not found under network = alpha-sepolia" ; "when account name not present")]
#[test_case("{\"alpha-sepolia\": {\"my_account\" : {}}}", "Error: Failed to parse field `alpha-sepolia.my_account` in file 'accounts.json': missing field `private_key`[..]" ; "when private key not present")]
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
    let args = apply_test_resource_bounds_flags(args);

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
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        Success: Account successfully deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.starkscan.co/tx/0x[..]
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

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        Success: Account successfully deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.starkscan.co/tx/0x[..]
    "});
}

pub async fn create_account(add_profile: bool, class_hash: &str, account_type: &str) -> TempDir {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: Value =
        serde_json::from_str(&contents).expect("Failed to parse accounts file at {path}");

    mint_token(
        items["alpha-sepolia"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        9_999_999_999_999_999_999_999_999_999_999,
    )
    .await;
    tempdir
}

#[test_case("oz"; "open_zeppelin_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
pub async fn test_happy_case_keystore(account_type: &str) {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = format!("my_account_{account_type}_undeployed_happy_case.json");

    copy_file(
        "tests/data/keystore/my_key.json",
        tempdir.path().join(keystore_file),
    );
    copy_file(
        format!("tests/data/keystore/{account_file}"),
        tempdir.path().join(&account_file),
    );
    set_keystore_password_env();

    let address = get_address_from_keystore(
        tempdir.path().join(keystore_file).to_str().unwrap(),
        tempdir.path().join(&account_file).to_str().unwrap(),
        KEYSTORE_PASSWORD_ENV_VAR,
        &account_type.parse().unwrap(),
    );

    mint_token(
        &address.into_hex_string(),
        9_999_999_999_999_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        &account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        Success: Account successfully deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.starkscan.co/tx/0x[..]
    "});

    let contents = fs::read_to_string(tempdir.path().join(account_file)).unwrap();
    let items: Value = serde_json::from_str(&contents).expect("Failed to parse accounts file at ");
    assert_eq!(items["deployment"]["status"], "deployed");
    assert!(!items["deployment"]["address"].is_null());
    assert!(items["deployment"]["salt"].is_null());
    assert!(items["deployment"]["context"].is_null());
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
    set_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account deploy
        Error: Account already deployed
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

    set_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account deploy
        Error: Public key and private key from keystore do not match
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
    set_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account deploy
        Error: Failed to find keystore file
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
    set_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account deploy
        Error: File containing the account does not exist: When using `--keystore` argument, the `--account` argument should be a path to the starkli JSON account file
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
    set_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account deploy
        Error: Failed to get status key from account JSON file
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
    set_keystore_password_env();

    let address = get_address_from_keystore(
        tempdir.path().join(keystore_file),
        tempdir.path().join(account_file),
        KEYSTORE_PASSWORD_ENV_VAR,
        &AccountType::OpenZeppelin,
    );

    mint_token(
        &address.into_hex_string(),
        9_999_999_999_999_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "some-name",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().stdout_matches(indoc! {r"
        Success: Account successfully deployed

        Transaction Hash: 0x[..]

        To see account deployment details, visit:
        transaction: https://sepolia.starkscan.co/tx/0x[..]
    "});
}
