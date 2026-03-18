use crate::e2e::account::create_account;
use crate::helpers::constants::{MAP_CONTRACT_DECLARE_TX_HASH_SEPOLIA, URL};
use crate::helpers::fixtures::get_transaction_hash;
use crate::helpers::runner::runner;
use conversions::string::IntoHexStr;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::OZ_CLASS_HASH;

const INVOKE_TX_HASH: &str = "0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1";

#[tokio::test]
async fn test_get_invoke_transaction() {
    let args = vec!["get", "tx", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args).env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1");
    let output = snapbox.assert().success();

    output.stdout_eq(indoc! {r"
        Success: Transaction found

        Type:                        INVOKE
        Version:                     3
        Transaction Hash:            0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1
        Sender Address:              0x01d091b30a2d20ca2509579f8beae26934bfdc3725c0b497f50b353b7a3c636f
        Nonce:                       98206
        Calldata:                    [0x1, 0x424ce41bea300e095e763d9fb4316af76c9da9c0fa926009f25b42b6f4ad04a, 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, 0x13, 0x441b0ab7fcd3923bd830e146e99ed90c4aebd19951eb6ed7b3713241aa8af, 0x29e701, 0xf29c0193adc354752489f1a7af2f507d72a5e5b76cce705094d05d72e21ab5, 0x6655cb7c, 0x304020100000000000000000000000000000000000000000000000000000000, 0x4, 0x27693e402, 0x276a2b3d2, 0x276a3f070, 0x276aeecf0, 0xb9eab07caffbd5538, 0x1, 0x2, 0x6771e459d1e5563ec13af0ca40f04406ff4b70e6cc9a534dce12957f46c0f24, 0x36383aebe2151145a66dd7a87d9c885a862339e35d2ee0bd9df4075d17a8979, 0x2cb74dff29a13dd5d855159349ec92f943bacf0547ff3734e7d84a15d08cbc5, 0xb1a29e2cfed2f0a9d5f137845280bb6ce746f2f4b6a2dd05ec794171f4012, 0x1f85c957582717816bd2c910ac678caf007f6f84d71bc5a95f38de0b6435163, 0x4225d1c8ee8e451a25e30c10689ef898e11ccf5c0f68d0fc7876c47b318e946]
        Account Deployment Data:     []
        Resource Bounds L1 Gas:      max_amount=39865, max_price_per_unit=226571933234745
        Resource Bounds L1 Data Gas: max_amount=0, max_price_per_unit=0
        Resource Bounds L2 Gas:      max_amount=0, max_price_per_unit=0
        Tip:                         0
        Paymaster Data:              []
        Nonce DA Mode:               L1
        Fee DA Mode:                 L1
        Signature:                   [0x1e45e75e772a8f21592cac2cb7d662ee53af236155621ae580bd02bbfa13bbc, 0x3528f5508bd323fa3301ca59823c0b35f17bc29a9a70d919be2ecd6f941d3db]
        
        To see transaction details, visit:
        transaction: https://sepolia.voyager.online/tx/0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1
    "});
}

#[tokio::test]
async fn test_get_invoke_transaction_alias_transaction() {
    let args = vec!["get", "transaction", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args).env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1");
    let output = snapbox.assert().success();

    assert_stdout_contains(output, "Success: Transaction found");
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

            Type:                        DEPLOY ACCOUNT
            Version:                     3
            Transaction Hash:            0x[..]
            Nonce:                       0
            Class Hash:                  0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564
            Contract Address Salt:       0x[..]
            Constructor Calldata:        [0x[..]]
            Resource Bounds L1 Gas:      max_amount=0, max_price_per_unit=1500000000
            Resource Bounds L1 Data Gas: max_amount=672, max_price_per_unit=1500000000
            Resource Bounds L2 Gas:      max_amount=1313760, max_price_per_unit=1500000000
            Tip:                         0
            Paymaster Data:              []
            Nonce DA Mode:               L1
            Fee DA Mode:                 L1
            Signature:                   [0x[..], 0x[..]]
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

    output.stdout_eq(indoc! {r"
        Success: Transaction found

        Type:                DECLARE
        Version:             2
        Transaction Hash:    0x04f644d3ea723b9c28781f2bea76e9c2cd8cc667b2861faf66b4e45402ea221c
        Sender Address:      0x0709cebece48663c3f0ece4b4553c9b1aaf325a3de5eb93792d5edfc3fdc42a8
        Nonce:               6
        Class Hash:          0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321
        Compiled Class Hash: 0x023ea170b0fc421a0ba919e32310cab42c16b3c9ded46add315a94ae63f5dde4
        Max Fee:             0x1559951f089bf
        Signature:           [0xb587f3ac9d32ea2ef741409681d8f255e300cbeb633e28a8557bcd1464f623, 0x600ddf65382e33485a9ed0cdb632233cf83a5e971c75e9dc9c31d585cec3655]
    "});
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
        Error: Transaction with provided hash was not found (does not exist)
        "},
    );
}

// TODO (#4258): Add test for invoke tx with `proof_facts`
