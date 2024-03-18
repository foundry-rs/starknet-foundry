use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH, URL};
use crate::helpers::fixtures::{copy_file, default_cli_args};
use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use indoc::indoc;

use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains, AsOutput};
use sncast::helpers::constants::CREATE_KEYSTORE_PASSWORD_ENV_VAR;
use std::{env, fs};
use tempfile::tempdir;

#[tokio::test]
pub async fn test_happy_case() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account_create_happy",
        "--salt",
        "0x1",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert();

    let stdout_str = output.as_stdout();
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(!stdout_str.contains("max_fee: 0x"));
    assert!(stdout_str.contains("address: 0x"));
    assert!(stdout_str.contains(
        "add_profile: --add-profile flag was not set. No profile added to snfoundry.toml"
    ));

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));
    assert!(contents.contains("class_hash"));
}

#[tokio::test]
pub async fn test_invalid_class_hash() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--class-hash",
        "0x10101",
        "--name",
        "my_account_create_happy",
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Class with hash 0x10101 is not declared, try using --class-hash with a hash of the declared class
        "},
    );
}

#[tokio::test]
pub async fn test_happy_case_generate_salt() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
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

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: [..]
        message: Account successfully created[..]
        "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));
    assert!(contents.contains("class_hash"));
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--add-profile",
        "my_account",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        "add_profile: Profile my_account successfully added to snfoundry.toml",
    );

    let contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains("[sncast.my_account]"));
    assert!(contents.contains("account = \"my_account\""));
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_already_exists() {
    let accounts_file = "accounts.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    copy_file(
        "tests/data/accounts/accounts.json",
        temp_dir.path().join(accounts_file),
    );
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: [..]
        message: Account successfully created[..]
        "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("deployed"));
}

#[tokio::test]
pub async fn test_profile_already_exists() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "myprofile",
        "--add-profile",
        "default",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Failed to add profile = default to the snfoundry.toml. Profile already exists
        "},
    );
}

#[tokio::test]
pub async fn test_account_already_exists() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "account",
        "create",
        "--name",
        "user1",
        "--salt",
        "0x1",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Account with name = user1 already exists in network with chain_id = SN_GOERLI
        "},
    );
}

#[tokio::test]
pub async fn test_happy_case_keystore() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key.json";
    let account_file = "my_account.json";
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: [..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));
}

#[tokio::test]
pub async fn test_happy_case_keystore_add_profile() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let keystore_file = "my_key.json";
    let account_file = "my_account.json";
    let accounts_json_file = "accounts.json";
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_file,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
        "--add-profile",
        "with_keystore",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    assert_stdout_contains(
        output,
        "add_profile: Profile with_keystore successfully added to snfoundry.toml",
    );

    let contents =
        fs::read_to_string(tempdir.path().join(account_file)).expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));

    let contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains(r"[sncast.with_keystore]"));
    assert!(contents.contains(r#"account = "my_account.json""#));
    assert!(!contents.contains(r#"accounts-file = "accounts.json""#));
    assert!(contents.contains(r#"keystore = "my_key.json""#));
    assert!(contents.contains(r#"url = "http://127.0.0.1:5055/rpc""#));
}

#[tokio::test]
pub async fn test_keystore_without_account() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key.json";

    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Argument `--account` must be passed and be a path when using `--keystore`
        "},
    );
}

#[tokio::test]
pub async fn test_keystore_file_already_exists() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_new.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        temp_dir.path().join(keystore_file),
    );
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Keystore file my_key.json already exists
        "},
    );
}

#[tokio::test]
pub async fn test_keystore_account_file_already_exists() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key_new.json";
    let account_file = "my_account.json";

    copy_file(
        "tests/data/keystore/my_account.json",
        temp_dir.path().join(account_file),
    );

    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account create
        error: Account file my_account.json already exists
        "},
    );
}

#[tokio::test]
pub async fn test_happy_case_keystore_int_format() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key_int.json";
    let account_file = "my_account_int.json";

    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "--int-format",
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: [..]
        max_fee: [..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));
}

#[tokio::test]
pub async fn test_happy_case_keystore_hex_format() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key_hex.json";
    let account_file = "my_account_hex.json";

    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "--hex-format",
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: 0x[..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));
}
