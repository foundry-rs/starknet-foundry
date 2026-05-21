use crate::helpers::constants::{
    DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_happy_case() {
    let args = vec![
        "get",
        "class-hash-at",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--url",
        URL,
    ];
    let snapbox = runner(&args).env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1");

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Class hash retrieved

        Class Hash: 0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321

        To see class details, visit:
        class: https://sepolia.voyager.online/class/0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321
    "});
}

#[tokio::test]
async fn test_with_block_id() {
    let args = vec![
        "get",
        "class-hash-at",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--block-id",
        "latest",
        "--url",
        URL,
    ];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Class hash retrieved

        Class Hash: 0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321
    "});
}

#[tokio::test]
async fn test_json_output() {
    let args = vec![
        "--json",
        "get",
        "class-hash-at",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["command"], "get class-hash-at");
    assert_eq!(json["type"], "response");
    assert_eq!(
        json["class_hash"],
        "0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321"
    );
}

#[tokio::test]
async fn test_nonexistent_contract_address() {
    let args = vec!["get", "class-hash-at", "0x0", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get class-hash-at
        Error: There is no contract at the specified address
        "},
    );
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec![
        "get",
        "class-hash-at",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--block-id",
        "invalid_block",
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get class-hash-at
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}
