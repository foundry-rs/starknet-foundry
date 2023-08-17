use crate::helpers::constants::{CONTRACTS_DIR, DEVNET_OZ_CLASS_HASH, URL};
use crate::helpers::fixtures::{default_cli_args, duplicate_directory_with_salt};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use std::fs;

#[tokio::test]
pub async fn test_happy_case() {
    let accounts_file = "./tmp/accounts.json";
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

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(stdout_str.contains("address: "));
    assert!(stdout_str
        .contains("add_profile: --add-profile flag was not set. No profile added to Scarb.toml"));

    let contents = fs::read_to_string(accounts_file).expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_generate_salt() {
    let accounts_file = "./tmp1/accounts.json";
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

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let current_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "1",
    ));
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
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&current_dir)
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("add_profile: Profile successfully added to Scarb.toml"));

    let contents =
        fs::read_to_string(current_dir.join("Scarb.toml")).expect("Unable to read Scarb.toml");
    assert!(contents.contains("[tool.sncast.my_account]"));
    assert!(contents.contains("account = \"my_account\""));

    fs::remove_dir_all(current_dir).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_already_exists() {
    let current_dir = Utf8PathBuf::from("./tmp2");
    let accounts_file = "./accounts.json";
    fs::create_dir_all(&current_dir).expect("Unable to create directory");

    fs_extra::file::copy(
        "tests/data/accounts/accounts.json",
        current_dir.join(accounts_file),
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
        .current_dir(&current_dir)
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: account create"));
    assert!(stdout_str.contains("max_fee: "));
    assert!(stdout_str.contains("address: "));

    let contents =
        fs::read_to_string(current_dir.join(accounts_file)).expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("deployed"));

    fs::remove_dir_all(current_dir).unwrap();
}

#[tokio::test]
pub async fn test_profile_already_exists() {
    let current_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "2",
    ));
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
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&current_dir)
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let std_err =
        std::str::from_utf8(&out.stderr).expect("failed to convert command stderr to string");
    assert!(std_err.contains(
        "error: Failed to add myprofile profile to the Scarb.toml. Profile already exists"
    ));

    fs::remove_dir_all(current_dir).unwrap();
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

    snapbox.assert().stderr_matches(indoc! {r#"
        command: account create
        error: Account with provided name already exists in network alpha-goerli
    "#});
}
