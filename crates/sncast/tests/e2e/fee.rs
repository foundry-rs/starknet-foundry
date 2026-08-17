use crate::helpers::constants::{ACCOUNT_FILE_PATH, MAP_CONTRACT_ADDRESS_SEPOLIA, URL};
use crate::helpers::fixtures::{create_and_deploy_oz_account, join_tempdirs};
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

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

#[test]
fn test_detailed_without_dry_run() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user11",
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
        "--detailed",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          --dry-run
        "},
    );
}

#[tokio::test]
async fn test_fee_params_from_config_are_used() {
    let account_dir = create_and_deploy_oz_account().await;
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_fee_params.toml", None);
    join_tempdirs(&account_dir, &config_dir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
    ];

    let output = runner(&args)
        .current_dir(config_dir.path())
        .assert()
        .failure();

    println!("{:?}", output.get_output().stderr);
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: invoke
        Error: The transaction's resources don't cover validation or the minimal transaction fee
        "},
    );
}

#[tokio::test]
async fn test_cli_max_fee_discards_config_fee_params() {
    let account_dir = create_and_deploy_oz_account().await;
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_fee_params.toml", None);
    join_tempdirs(&account_dir, &config_dir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
        // Discards all the bounds from the config, which are too low to pay for the tx
        "--max-fee",
        "1000000000000000000000000",
    ];

    let output = runner(&args)
        .current_dir(config_dir.path())
        .assert()
        .success();

    assert_stdout_contains(output, "Success: Invoke completed");
}
