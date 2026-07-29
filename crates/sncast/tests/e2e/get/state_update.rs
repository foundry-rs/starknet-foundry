use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[tokio::test]
async fn test_happy_case() {
    let args = vec!["get", "state-update", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: State update retrieved

            Old Root: 0x[..]
        "},
    );
}

#[tokio::test]
async fn test_happy_case_with_block_id() {
    let args = vec!["get", "state-update", "latest", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: State update retrieved

            Block Hash:[..]0x[..]
            Old Root:[..]0x[..]
            New Root:[..]0x[..]
        "},
    );
}

#[tokio::test]
async fn test_happy_case_json() {
    let args = vec!["--json", "get", "state-update", "latest", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        r#"[..]"command":"get state-update"[..]"new_root":"0x[..]"[..]"state_diff":{[..]}[..]"type":"response"}"#,
    );
}

#[tokio::test]
async fn test_invalid_block_id() {
    let args = vec!["get", "state-update", "invalid_block", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get state-update
        Error: Incorrect value passed for block_id = invalid_block. Possible values are `pre_confirmed`, `latest`, block hash (hex) and block number (u64)
        "},
    );
}

#[tokio::test]
async fn test_nonexistent_block() {
    let args = vec!["get", "state-update", "0x123", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get state-update
        Error: Block was not found
        "},
    );
}
