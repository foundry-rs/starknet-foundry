use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_happy_case() {
    let args = vec!["get", "block", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Block retrieved

        Status:[..]Pre confirmed
        Block Number:[..]
        Timestamp:[..]
        Sequencer Address:[..]0x[..]
        L1 Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L2 Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L1 Data Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L1 DA Mode:[..]
        Starknet Version:[..]
        Transaction Count:[..]
        Transactions:[..]
    "});
}

#[tokio::test]
async fn test_happy_case_with_block_id() {
    let args = vec!["get", "block", "latest", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Block retrieved

        Status:[..]
        Block Hash:[..]0x[..]
        Block Number:[..]
        Parent Hash:[..]0x[..]
        New Root:[..]0x[..]
        Timestamp:[..]
        Sequencer Address:[..]0x[..]
        L1 Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L2 Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L1 Data Gas Price:[..]price_in_fri=[..], price_in_wei=[..]
        L1 DA Mode:[..]
        Starknet Version:[..]
        Transaction Count:[..]
        Transactions:[..]
    "});
}

#[tokio::test]
async fn test_happy_case_json() {
    let args = vec!["--json", "get", "block", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        {"block_number":[..],"command":"get block","l1_da_mode":"[..]","l1_data_gas_price":{"price_in_fri":"0x[..]","price_in_wei":"0x[..]"},"l1_gas_price":{"price_in_fri":"0x[..]","price_in_wei":"0x[..]"},"l2_gas_price":{"price_in_fri":"0x[..]","price_in_wei":"0x[..]"},"sequencer_address":"0x[..]","starknet_version":"[..]","timestamp":[..],"transactions":[[..]],"type":"response"}
    "#});
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec!["get", "block", "invalid_block", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get block
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}

#[tokio::test]
async fn test_nonexistent_block() {
    let args = vec!["get", "block", "0x123", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get block
        Error: Block was not found
        ",
        },
    );
}
