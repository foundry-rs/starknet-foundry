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
async fn test_happy_case_exact_values() {
    let args = vec!["get", "state-update", "12626869", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: State update retrieved

            Block Hash:         0x045705766be3a89746b06ed6d263b86c2c22caa87819053a4431c3ffc964e006
            Old Root:           0x012583e82ebd72c9b802366b39c679d8a2bfa207075071ff314dda9c53a52eff
            New Root:           0x02ac1a8a16a2d8390e651d3a41d77fee34d669d4cb7f49a3e7cd7fdae4384b19

            Storage Diffs
              Contract Address: 0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
                Key:            0x05906818c3dbf002df7c4e3fa8cef5c7835f58dd45a12a8ebc71c0e30471a054
                Value:          0x00000000000000000000000000000000000000000010008cf787280cc6d573c6
                Key:            0x05496768776e3db30053404f18067d81a6e06f5a2b0de326e21298fd9d569a9a
                Value:          0x0000000000000000000000000000000000000000000420155261135a9c73c807
              Contract Address: 0x0000000000000000000000000000000000000000000000000000000000000001
                Key:            0x0000000000000000000000000000000000000000000000000000000000c0abab
                Value:          0x015f14efa2747fb9f715c1a7de3d38186e2b930f243ffe6ddad837bc76a6470a

            Nonces
              Contract Address: 0x015569a4dae53e13da0b0f9332d88539c96db79858b14fb15e571a9f46b6c1be
              Nonce:            0x47864b
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
