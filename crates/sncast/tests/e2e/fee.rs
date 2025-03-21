use crate::helpers::constants::{ACCOUNT_FILE_PATH, MAP_CONTRACT_ADDRESS_SEPOLIA, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[test]
fn test_max_fee_used_with_other_args() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user11",
        "--wait",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "0x2",
        "--max-fee",
        "1",
        "--l1-gas",
        "1",
        "--l1-gas-price",
        "1",
        "--l2-gas",
        "1",
        "--l2-gas-price",
        "1",
        "--l1-data-gas",
        "1",
        "--l1-data-gas-price",
        "1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--max-fee <MAX_FEE>' cannot be used with:
          --l1-gas <L1_GAS>
          --l1-gas-price <L1_GAS_PRICE>
          --l2-gas <L2_GAS>
          --l2-gas-price <L2_GAS_PRICE>
          --l1-data-gas <L1_DATA_GAS>
          --l1-data-gas-price <L1_DATA_GAS_PRICE>
        "},
    );
}
