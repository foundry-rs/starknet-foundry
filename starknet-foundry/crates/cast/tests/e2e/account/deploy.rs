use crate::helpers::constants::{CONTRACTS_DIR, NETWORK, URL};
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_transaction_hash, get_transaction_receipt, mint_token,
};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::DeployAccount;
use std::fs;

#[tokio::test]
pub async fn test_happy_case() {
    // setup
    let accounts_file = "./tmp3/accounts.json";
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

    runner(&args).assert().success();

    let contents = fs::read_to_string(accounts_file).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");

    mint_token(
        items["alpha-goerli"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        1e17,
    )
    .await;

    // test
    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "10000000000000000",
    ];

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let hash = get_transaction_hash(&out.stdout);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, DeployAccount(_)));

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("Deploy account"));
    assert!(stdout_str.contains("transaction_hash"));

    let contents = fs::read_to_string(accounts_file).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");
    assert_eq!(items["alpha-goerli"]["my_account"]["deployed"], true);

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    // setup
    let created_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "3",
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

    Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(&args)
        .assert()
        .success();

    let contents = fs::read_to_string(created_dir.join(accounts_file)).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");

    mint_token(
        items["alpha-goerli"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        1e17,
    )
    .await;

    // test
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
        "10000000000000000",
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
    assert!(stdout_str.contains("Deploy account"));
    assert!(stdout_str.contains("transaction_hash"));

    fs::remove_dir_all(created_dir.join(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
async fn test_empty_accounts_file() {
    let created_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "4",
    ));
    let accounts_file = "./accounts.json";
    fs::write(created_dir.join(accounts_file), "{}").unwrap();

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
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
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        dbg!(std::str::from_utf8(&out.stderr).expect("failed to convert command output to string"));
    assert!(stderr_str.contains("error: Provided network does not have any accounts defined"));

    fs::remove_dir_all(created_dir.join(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
async fn test_account_name_not_present() {
    let created_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "5",
    ));
    let accounts_file = "./accounts.json";
    fs::write(created_dir.join(accounts_file), "{\"alpha-goerli\": {}}").unwrap();

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
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
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        dbg!(std::str::from_utf8(&out.stderr).expect("failed to convert command output to string"));
    assert!(stderr_str.contains("error: Account with provided name does not exist"));

    fs::remove_dir_all(created_dir.join(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
async fn test_private_key_not_present() {
    let created_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "6",
    ));
    let accounts_file = "./accounts.json";
    fs::write(
        created_dir.join(accounts_file),
        "{\"alpha-goerli\": {\"my_account\" : {}}}",
    )
    .unwrap();

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
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
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        dbg!(std::str::from_utf8(&out.stderr).expect("failed to convert command output to string"));
    assert!(stderr_str.contains("error: Couldn't get private key from accounts file"));

    fs::remove_dir_all(created_dir.join(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
async fn test_salt_not_present() {
    let created_dir = Utf8PathBuf::from(duplicate_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/v1/balance",
        "put",
        "7",
    ));
    let accounts_file = "./accounts.json";
    fs::write(created_dir.join(accounts_file), "{\"alpha-goerli\": {\"my_account\" : {\"private_key\": \"0x5c0883893a3c32b57769f168383c53ee\"}}}").unwrap();

    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
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
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        dbg!(std::str::from_utf8(&out.stderr).expect("failed to convert command output to string"));
    assert!(stderr_str.contains("error: Couldn't get salt from accounts file"));

    fs::remove_dir_all(created_dir.join(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
async fn test_too_low_max_fee() {
    // setup
    let accounts_file = "./tmp4/accounts.json";
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

    runner(&args).assert().success();

    let contents = fs::read_to_string(accounts_file).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");

    mint_token(
        items["alpha-goerli"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        1e17,
    )
    .await;

    // test
    let args = vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        accounts_file,
        "account",
        "deploy",
        "--name",
        "my_account",
        "--max-fee",
        "1",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Transaction has been rejected
    "#});

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}
