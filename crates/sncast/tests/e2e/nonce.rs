use crate::helpers::constants::{DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_happy_case() {
    let args = vec![
        "get",
        "nonce",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Nonce retrieved

        Nonce: 0x[..]
    "});
}

#[tokio::test]
async fn test_happy_case_with_block_id() {
    let args = vec![
        "get",
        "nonce",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--block-id",
        "latest",
        "--url",
        URL,
    ];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Nonce retrieved

        Nonce: 0x[..]
    "});
}

#[tokio::test]
async fn test_happy_case_json() {
    let args = vec![
        "--json",
        "get",
        "nonce",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        {"command":"get nonce","nonce":"0x[..]","type":"response"}
    "#});
}

#[tokio::test]
async fn test_nonexistent_contract_address() {
    let args = vec!["get", "nonce", "0x0", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get nonce
        Error: There is no contract at the specified address
        "},
    );
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec![
        "get",
        "nonce",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--block-id",
        "invalid_block",
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get nonce
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}
