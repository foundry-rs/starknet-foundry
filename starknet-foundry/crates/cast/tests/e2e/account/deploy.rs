use crate::helpers::constants::{CONTRACTS_DIR, NETWORK, URL};
use crate::helpers::fixtures::{
    duplicate_directory_with_salt, get_transaction_hash, get_transaction_receipt, mint_token,
};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
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
        .args(args);

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
