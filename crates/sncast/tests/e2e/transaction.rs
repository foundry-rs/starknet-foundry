use crate::e2e::account::create_account;
use crate::helpers::constants::{MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA, URL};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::get_transaction_hash;
use crate::helpers::runner::runner;
use conversions::string::IntoHexStr;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::OZ_CLASS_HASH;

const INVOKE_TX_HASH: &str = "0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1";

#[tokio::test]
async fn test_invoke_transaction() {
    let args = vec!["get", "tx", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: Transaction found

            Type:[..]INVOKE
            Version:[..]3
            Transaction Hash:[..]0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1
            Sender Address:[..]0x[..]
            Nonce:[..]0x[..]
            Calldata:[..][0x[..]]
            Account Deployment Data:[..][]
            Resource Bounds L1 Gas:[..]max_amount=0x[..], max_price_per_unit=0x[..]
            Resource Bounds L1 Data Gas:[..]max_amount=0x0, max_price_per_unit=0x0
            Resource Bounds L2 Gas:[..]max_amount=0x0, max_price_per_unit=0x0
            Tip:[..]
            Paymaster Data:[..][]
            Nonce DA Mode:[..]L1
            Fee DA Mode:[..]L1
            Signature:[..][0x[..]]
        "},
    );
}

#[tokio::test]
async fn test_json_output() {
    let args = vec!["--json", "get", "tx", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["command"], "get tx");
    assert_eq!(json["type"], "response");
    assert_eq!(json["transaction_type"], "INVOKE_V3");

    let tx = &json["transaction"];
    assert!(tx["transaction_hash"].as_str().unwrap().starts_with("0x"));
    assert!(tx["sender_address"].as_str().unwrap().starts_with("0x"));
    assert!(tx["nonce"].as_str().unwrap().starts_with("0x"));
    assert!(tx["calldata"].is_array());
    assert!(tx["signature"].is_array());
    assert!(tx["tip"].as_str().unwrap().starts_with("0x"));
    assert!(tx["paymaster_data"].is_array());
    assert_eq!(tx["nonce_data_availability_mode"], "L1");
    assert_eq!(tx["fee_data_availability_mode"], "L1");
    assert!(tx["account_deployment_data"].is_array());
}

#[tokio::test]
async fn test_deploy_account_transaction() {
    let tempdir = create_account(false, &OZ_CLASS_HASH.into_hex_string(), "oz").await;
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let hash = get_transaction_hash(&output.get_output().stdout);
    let hash_hex = format!("{hash:#x}");

    let get_tx_args = vec!["get", "tx", hash_hex.as_str(), "--url", URL];
    let get_tx_output = runner(&get_tx_args)
        .current_dir(tempdir.path())
        .assert()
        .success();

    assert_stdout_contains(
        get_tx_output,
        indoc! {r"
            Success: Transaction found

            Type:[..]DEPLOY ACCOUNT
            Version:[..]
            Transaction Hash:[..]
            Nonce:[..]
            Class Hash:[..]
        "},
    );
}

#[tokio::test]
async fn test_declare_transaction() {
    let args = vec![
        "get",
        "tx",
        MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: Transaction found

            Type:[..]DECLARE
            Version:[..]2
            Transaction Hash:[..]0x04f644d3ea723b9c28781f2bea76e9c2cd8cc667b2861faf66b4e45402ea221c
            Sender Address:[..]0x0709cebece48663c3f0ece4b4553c9b1aaf325a3de5eb93792d5edfc3fdc42a8
            Nonce:[..]0x6
            Class Hash:[..]0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321
            Compiled Class Hash:[..]0x023ea170b0fc421a0ba919e32310cab42c16b3c9ded46add315a94ae63f5dde4
            Max Fee:[..]0x1559951f089bf
            Signature:[..][0xb587f3ac9d32ea2ef741409681d8f255e300cbeb633e28a8557bcd1464f623, 0x600ddf65382e33485a9ed0cdb632233cf83a5e971c75e9dc9c31d585cec3655]
        "},
    );
}

#[tokio::test]
async fn test_explorer_link() {
    let args = vec!["get", "tx", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args).env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1");
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Success: Transaction found

            Type:[..]INVOKE
            Version:[..]3
            Transaction Hash:[..]0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1

            To see transaction details, visit:
            transaction: [..]
        "},
    );
}

#[tokio::test]
async fn test_nonexistent_transaction() {
    let args = vec!["get", "tx", "0x1", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get tx
        Error: Failed to get transaction: Transaction with provided hash was not found (does not exist)
        "},
    );
}
