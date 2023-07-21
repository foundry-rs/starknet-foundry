use crate::helpers::constants::{CONTRACTS_DIR, NETWORK, URL};
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
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: Create account"));
    assert!(stdout_str.contains("message: Account successfully created. Prefund generated address with at least 432300000000 tokens. It is good to send more in the case of higher demand, max_fee * 2 = 864600000000"));
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
pub async fn test_happy_case_generate_salt() {
    let accounts_file = "./tmp2/accounts.json";
    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
    ];

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("command: Create account"));
    assert!(stdout_str.contains("message: Account successfully created. Prefund generated address with at least 432300000000 tokens. It is good to send more in the case of higher demand, max_fee * 2 = 864600000000"));
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
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--add-profile",
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
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "myprofile",
        "--add-profile",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&current_dir)
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command stderr to string");
    assert!(stdout_str.contains(
        "add_profile: Failed to add myprofile profile to the Scarb.toml. Profile already exists"
    ));
    assert!(stdout_str.contains("command: Create account"));
    assert!(stdout_str.contains("message: Account successfully created. Prefund generated address with at least 432300000000 tokens. It is good to send more in the case of higher demand, max_fee * 2 = 864600000000"));

    fs::remove_dir_all(current_dir).unwrap();
}

#[tokio::test]
pub async fn test_account_already_exists() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "account", "create", "--name", "user1", "--salt", "0x1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        error: Account with provided name already exists in this network
    "#});
}
