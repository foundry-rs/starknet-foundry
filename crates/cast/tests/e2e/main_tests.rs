use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, MAP_CONTRACT_ADDRESS_V1, NETWORK, URL,
};
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_happy_case_from_scarb() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--path-to-scarb-toml",
        "tests/data/files/correct_Scarb.toml",
        "call",
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: call
        error: There is no contract at the specified address
    "#});
}

#[tokio::test]
async fn test_happy_case_from_cli_no_scarb() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--url",
        URL,
        "--network",
        NETWORK,
        "--account",
        ACCOUNT,
        "call",
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        command: call
        error: There is no contract at the specified address
    "#});
}

#[tokio::test]
async fn test_happy_case_from_cli_with_scarb() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--json",
        "--path-to-scarb-toml",
        "tests/data/files/correct_Scarb.toml",
        "--profile",
        "profile1",
        "--url",
        URL,
        "--network",
        NETWORK,
        "--account",
        ACCOUNT,
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_V1,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
{
  "command": "call",
  "response": "[0x0]"
}
"#});
}

#[tokio::test]
async fn test_happy_case_mixed() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--json",
        "--path-to-scarb-toml",
        "tests/data/files/correct_Scarb.toml",
        "--profile",
        "profile2",
        "--account",
        ACCOUNT,
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_V1,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
{
  "command": "call",
  "response": "[0x0]"
}
"#});
}

#[tokio::test]
async fn test_missing_account() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--url",
        URL,
        "--network",
        NETWORK,
        "declare",
        "--contract-name",
        "whatever",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        Error: Account name not passed nor found in Scarb.toml
    "#});
}

#[tokio::test]
async fn test_missing_network() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--url",
        URL,
        "--account",
        ACCOUNT,
        "declare",
        "--contract-name",
        "whatever",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        Error: Network not passed nor found in Scarb.toml
    "#});
}

#[tokio::test]
async fn test_missing_url() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--network",
        NETWORK,
        "--account",
        ACCOUNT,
        "declare",
        "--contract-name",
        "whatever",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        Error: RPC url not passed nor found in Scarb.toml
    "#});
}
