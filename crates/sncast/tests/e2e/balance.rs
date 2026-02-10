use crate::helpers::constants::URL;
use crate::helpers::fixtures::get_accounts_path;
use crate::helpers::runner::runner;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::AsOutput;
use sncast::helpers::token::Token;
use tempfile::tempdir;
use test_case::test_case;

#[tokio::test]
pub async fn happy_case() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "balance-test",
        "balance",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r"
        Balance: 109394843313476728397 fri
    "});
}

#[tokio::test]
pub async fn happy_case_json() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--json",
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r#"
        {"balance":"[..]","command":"balance","token_unit":"fri","type":"response"}
    "#});
}

#[test_case(&Token::Strk)]
#[test_case(&Token::Eth)]
#[tokio::test]
pub async fn happy_case_with_token(token: &Token) {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let token_unit = token.as_token_unit();
    let token = token.to_string();
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--token",
        &token,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(formatdoc! {r"
        Balance: [..] {token_unit}
    "});
}

#[tokio::test]
pub async fn happy_case_with_block_id() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--block-id",
        "latest",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r"
        Balance: [..] fri
    "});
}

#[tokio::test]
pub async fn invalid_token() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--token",
        "xyz",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_eq(indoc! {r"
        error: invalid value 'xyz' for '--token <TOKEN>'
          [possible values: strk, eth]

        For more information, try '--help'.
    "});
}

#[tokio::test]
pub async fn happy_case_with_token_address() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let strk_address = "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--token-address",
        strk_address,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r"
        Balance: [..]
    "});
}

#[tokio::test]
pub async fn happy_case_json_with_token_address() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--json",
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let balance_str = std::str::from_utf8(&output.get_output().stdout).unwrap();

    assert!(!balance_str.contains("0x"));

    output.stdout_eq(indoc! {r#"
        {"balance":"[..]"}
    "#});
}

#[tokio::test]
pub async fn nonexistent_token_address() {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "balance",
        "--token-address",
        "0x123",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let snapbox = snapbox.assert().failure();
    let err = snapbox.as_stderr();
    assert!(err.contains("Error: There is no contract at the specified address"));
}
