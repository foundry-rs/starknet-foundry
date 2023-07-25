use crate::helpers::constants::{NETWORK, URL};
use crate::helpers::fixtures::{create_account, get_transaction_hash, get_transaction_receipt};
use camino::Utf8PathBuf;
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use starknet::core::types::TransactionReceipt::DeployAccount;
use std::fs;
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let (created_dir, accounts_file) = create_account("3", false).await;

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

    let contents = fs::read_to_string(created_dir.join(accounts_file)).unwrap();
    let items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");
    assert_eq!(items["alpha-goerli"]["my_account"]["deployed"], true);

    fs::remove_dir_all(created_dir).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let (created_dir, accounts_file) = create_account("4", true).await;

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

    fs::remove_dir_all(created_dir).unwrap();
}

#[test_case("4", "{}", "error: Provided network does not have any accounts defined" ; "when empty file")]
#[test_case("5", "{\"alpha-goerli\": {}}", "error: Account with provided name does not exist" ; "when account name not present")]
#[test_case("6", "{\"alpha-goerli\": {\"my_account\" : {}}}", "error: Couldn't get private key from accounts file" ; "when private key not present")]
#[test_case("7", "{\"alpha-goerli\": {\"my_account\" : {\"private_key\": \"0x1\"}}}", "error: Couldn't get salt from accounts file" ; "when salt not present")]
fn test_account_deploy_error(salt: &str, accounts_content: &str, error: &str) {
    let current_dir = Utf8PathBuf::from("./tmp".to_string() + salt);
    fs::create_dir_all(&current_dir).expect("Unable to create directory");
    let accounts_file = "./accounts.json";
    fs::write(current_dir.join(accounts_file), accounts_content).unwrap();

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
        .current_dir(&current_dir)
        .args(args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        dbg!(std::str::from_utf8(&out.stderr).expect("failed to convert command output to string"));
    assert!(stderr_str.contains(error));

    fs::remove_dir_all(current_dir).unwrap();
}

#[tokio::test]
async fn test_too_low_max_fee() {
    let (created_dir, accounts_file) = create_account("5", false).await;

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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(&created_dir)
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Transaction has been rejected
    "#});

    fs::remove_dir_all(created_dir).unwrap();
}
