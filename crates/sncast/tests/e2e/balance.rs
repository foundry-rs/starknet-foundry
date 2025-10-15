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
        "user1",
        "balance",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        Account Address: 0x[..]
        Balance:         [..] strk
    "});
}

#[test_case(&Token::Strk)]
#[test_case(&Token::Eth)]
#[tokio::test]
pub async fn happy_case_with_token(token: &Token) {
    let tempdir = tempdir().unwrap();
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

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

    snapbox.assert().stdout_matches(formatdoc! {r"
        Account Address: 0x[..]
        Balance:         [..] {token}
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

    snapbox.assert().stdout_matches(indoc! {r"
        Account Address: 0x[..]
        Balance:         [..] strk
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

    snapbox.assert().stderr_matches(indoc! {r"
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

    snapbox.assert().stdout_matches(indoc! {r"
        Account Address: 0x[..]
        Balance:         [..]
    "});
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
    println!("Error: {}", err);
    assert!(err.contains("Error: Error: There is no contract at the specified address"));
}
