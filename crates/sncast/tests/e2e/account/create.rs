use crate::helpers::constants::{DEVNET_OZ_CLASS_HASH, URL};
use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use sncast::helpers::configuration::copy_config_to_tempdir;
use sncast::helpers::constants::CREATE_KEYSTORE_PASSWORD_ENV_VAR;
use std::{env, fs};
use tempfile::TempDir;
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let accounts_file = "./tmp-c1/accounts.json";
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

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(stdout_str.contains("address: "));
    assert!(stdout_str.contains(
        "add_profile: --add-profile flag was not set. No profile added to snfoundry.toml"
    ));

    let contents = fs::read_to_string(accounts_file).expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));
    assert!(contents.contains("class_hash"));

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_generate_salt() {
    let accounts_file = "./tmp-c2/accounts.json";
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

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(stdout_str.contains("address: "));

    let contents = fs::read_to_string(accounts_file).expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));
    assert!(contents.contains("class_hash"));

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = TempDir::new().expect("Failed to create a temporary directory");
    let accounts_file = "./accounts.json";

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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(
        stdout_str.contains("add_profile: Profile my_account successfully added to snfoundry.toml")
    );

    let contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains("[sncast.my_account]"));
    assert!(contents.contains("account = \"my_account\""));
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_already_exists() {
    let accounts_file = "./accounts.json";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    fs_extra::file::copy(
        "tests/data/accounts/accounts.json",
        temp_dir.path().join(accounts_file),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy accounts.json");

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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(stdout_str.contains("address: "));

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("deployed"));
}

#[tokio::test]
pub async fn test_profile_already_exists() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let accounts_file = "./accounts.json";

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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let std_err =
        std::str::from_utf8(&out.stderr).expect("failed to convert command stderr to string");
    assert!(std_err.contains(
        "error: Failed to add profile = default to the snfoundry.toml. Profile already exists"
    ));
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

    snapbox.assert().stderr_matches(indoc! {r"
        command: account create
        error: Account with name = user1 already exists in network with chain_id = SN_GOERLI
    "});
}

#[tokio::test]
pub async fn test_happy_case_keystore() {
    let keystore_path = "my_key.json";
    let account_path = "my_account.json";
    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: [..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(account_path).expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));

    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
}

#[tokio::test]
pub async fn test_happy_case_keystore_add_profile() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let keystore_path = "my_key.json";
    let account_path = "my_account.json";
    let accounts_file = "accounts.json";
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
        "--add-profile",
        "with_keystore",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    let bdg = snapbox.assert().success();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str
        .contains("add_profile: Profile with_keystore successfully added to snfoundry.toml"));

    let contents =
        fs::read_to_string(tempdir.path().join(account_path)).expect("Unable to read created file");
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
    let keystore_path = "my_key.json";
    _ = fs::remove_file(keystore_path);
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: account create
        error: Argument `--account` must be passed and be a path when using `--keystore`
    "});

    _ = fs::remove_file(keystore_path);
}

#[test_case("tests/data/keystore/my_key.json", "tests/data/keystore/my_account_new.json", "error: Keystore file tests/data/keystore/my_key.json already exists" ; "when keystore exists")]
#[test_case("tests/data/keystore/my_key_new.json", "tests/data/keystore/my_account.json", "error: Account file tests/data/keystore/my_account.json already exists" ; "when account exists")]
pub fn test_keystore_already_exists(keystore_path: &str, account_path: &str, error: &str) {
    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "create",
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
pub async fn test_happy_case_keystore_int_format() {
    let keystore_path = "my_key_int.json";
    let account_path = "my_account_int.json";
    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "--int-format",
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: [..]
        max_fee: [..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(account_path).expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));

    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
}

#[tokio::test]
pub async fn test_happy_case_keystore_hex_format() {
    let keystore_path = "my_key_hex.json";
    let account_path = "my_account_hex.json";
    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
    env::set_var(CREATE_KEYSTORE_PASSWORD_ENV_VAR, "123");

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "--hex-format",
        "account",
        "create",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = runner(&args);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account create
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
        address: 0x[..]
        max_fee: 0x[..]
        message: Account successfully created[..]
    "});

    let contents = fs::read_to_string(account_path).expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));

    _ = fs::remove_file(keystore_path);
    _ = fs::remove_file(account_path);
}
