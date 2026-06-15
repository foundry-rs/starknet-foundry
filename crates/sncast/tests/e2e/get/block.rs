use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

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
async fn test_full_flag() {
    let args = vec!["get", "block", "latest", "--full", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    // With `--full`, transactions are rendered with their full details
    // instead of being listed as a flat list of hashes.
    assert_stdout_contains(
        output,
        indoc! {r"
            Transaction Count:[..]

            Transaction #1
              Type:[..]
              [..]Transaction Hash:[..]0x[..]
        "},
    );
}

#[tokio::test]
async fn test_full_flag_json() {
    let args = vec!["--json", "get", "block", "latest", "--full", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    // The transactions field holds full transaction objects, not just hashes.
    assert_stdout_contains(output, r#"[..]"transactions":[{[..]}],"type":"response"}"#);
}

#[tokio::test]
async fn test_full_flag_exact_values() {
    let args = vec!["get", "block", "7776130", "--full", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: Block retrieved

            Status:                        Accepted on L1
            Block Hash:                    0x000ea0b5f1ac9d9c9c354085feddd9995a9890272d5a9a3c12e13188c2a22602
            Block Number:                  7776130
            Parent Hash:                   0x0698a23898725b00868fe0f1e0cc8fd352c8492027bd44d3d930f8c1f48db897
            New Root:                      0x07062678fce28f7309bc651840dc0cf3791660581a9a0f6cf7359afebf073f66
            Timestamp:                     1773825600
            Sequencer Address:             0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8
            L1 Gas Price:                  price_in_fri=58068665447298, price_in_wei=1000000022
            L2 Gas Price:                  price_in_fri=8000000000, price_in_wei=137767
            L1 Data Gas Price:             price_in_fri=851867303, price_in_wei=14670
            L1 DA Mode:                    BLOB
            Starknet Version:              0.14.1
            Transaction Count:             4

            Transaction #1
              Type:                        INVOKE
              Version:                     3
              Transaction Hash:            0x044eee159a2983e017adfe9dc84908b9f6ba0e938c9e30c0ed613fe72cf5dea3
              Sender Address:              0x05677b3b308df6821be9a3c10423db8d9f7ff4d5bef09e32d973c8c89bf5683b
              Nonce:                       4425792
              Calldata:                    [0x2, 0x40e87b7fab5199ed3597b28b21f07e941d309bae7d465ada74e1b97e3d31f9a, 0x1a8e87e9d2008fcd3ce423ae5219c21e49be18d05d72825feb7e2bb687ba35c, 0x2, 0x6762e02bbcb35db2ecfefd4a786a453b, 0x5afb06446f33659903140e1c4accf0a7, 0x40e87b7fab5199ed3597b28b21f07e941d309bae7d465ada74e1b97e3d31f9a, 0x27a4a7332e590dd789019a6d125ff2aacd358e453090978cbf81f0d85e4c045, 0x2, 0x352, 0x1bf4e01fa2e9a2893340b16cad4eeed0034852382edce3ed9f232c8f96678c0]
              Account Deployment Data:     []
              Resource Bounds L1 Gas:      max_amount=70000, max_price_per_unit=2488849860263936
              Resource Bounds L1 Data Gas: max_amount=10000, max_price_per_unit=27659894942675796
              Resource Bounds L2 Gas:      max_amount=100000000, max_price_per_unit=50000000000
              Tip:                         100000000
              Paymaster Data:              []
              Nonce DA Mode:               L1
              Fee DA Mode:                 L1
              Signature:                   [0x5711b89412ec224a787db9f9485c98c9dccc6a1ca48d53c92d729bd99615661, 0xd0805f3dda4eacc2b6773b4833d8e1c923f15eb653bbebbd26c8437973b8aa]

            Transaction #2
              Type:[..]
              [..]Transaction Hash:[..]0x[..]

            Transaction #3
              Type:[..]
              [..]Transaction Hash:[..]0x[..]

            Transaction #4
              Type:[..]
              [..]Transaction Hash:[..]0x[..]
        "},
    );
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
