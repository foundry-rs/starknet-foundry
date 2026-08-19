use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use serde_json::{Value, json};
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TRANSACTION_HASH: &str = "0xabc";
const INVOKE_TX_HASH: &str = "0x26476da48e56e5e7025543ad0bb9105df00ee08571c6d17c4207462ff7717c4";
const DECLARE_TX_HASH: &str = "0x6054540622d534ffffb162a0e80c21bc106581eafeb3efad29385b78e04983d";
const DEPLOY_ACCOUNT_TX_HASH: &str =
    "0x06718b783a0b888f5421c4eb76a532feb9fd5167b2b09274298f79798c782b32";
const L1_HANDLER_TX_HASH: &str = "0x4c8c57b3ab646ef56aef3def69a01bc86d049b98f25ebfe3699334d86c24d5";
const REVERTED_INVOKE_TX_HASH: &str =
    "0x00fecca6a328dd11f40b79c30fe22d23bc6975d1a0923a95b90aff4016a84333";

fn invocation() -> Value {
    json!({
        "contract_address": "0x123",
        "entry_point_selector": "0x240060cdb34fcc260f41eac7474ee1d7c80b7e3607daff9ac67c7ea2ebb1c44",
        "calldata": ["0x7"],
        "caller_address": "0x0",
        "class_hash": "0x456",
        "entry_point_type": "EXTERNAL",
        "call_type": "CALL",
        "result": ["0x9"],
        "calls": [],
        "events": [],
        "messages": [],
        "execution_resources": { "l1_gas": 0, "l2_gas": 0 },
        "is_reverted": false
    })
}

fn trace() -> Value {
    json!({
        "type": "INVOKE",
        "validate_invocation": null,
        "execute_invocation": invocation(),
        "fee_transfer_invocation": null,
        "state_diff": null,
        "execution_resources": { "l1_gas": 0, "l1_data_gas": 0, "l2_gas": 0 }
    })
}

async fn mock_trace(mock_server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({ "method": "starknet_specVersion" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0.10.0"
        })))
        .expect(1)
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(body_partial_json(json!({
            "method": "starknet_traceTransaction",
            "params": { "transaction_hash": TRANSACTION_HASH }
        })))
        .respond_with(response)
        .expect(1)
        .mount(mock_server)
        .await;
}

async fn mock_contract_class(mock_server: &MockServer, abi: Value) {
    Mock::given(method("POST"))
        .and(body_partial_json(json!({ "method": "starknet_getClass" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "sierra_program": [],
                "contract_class_version": "0.1.0",
                "entry_points_by_type": {
                    "CONSTRUCTOR": [],
                    "EXTERNAL": [],
                    "L1_HANDLER": []
                },
                "abi": abi.to_string()
            }
        })))
        .expect(1)
        .mount(mock_server)
        .await;
}

#[tokio::test]
async fn test_invoke_transaction_trace() {
    runner(&["get", "tx-trace", INVOKE_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                     INVOKE
        Transaction Hash:         0x026476da48e56e5e7025543ad0bb9105df00ee08571c6d17c4207462ff7717c4
        Validate Invocation
          Entry Point Selector:   __validate__
          Contract Address:       0x0350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
          Calldata:               array![Call { to: ContractAddress(0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x2, 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2, 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711].span() }]
          Result:                 success: 0x56414c4944
        Execute Invocation
          Entry Point Selector:   __execute__
          Contract Address:       0x0350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
          Calldata:               array![Call { to: ContractAddress(0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0), selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, calldata: array![0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x2, 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2, 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711].span() }]
          Result:                 success: array![array![].span()]
          Calls
            Entry Point Selector: transmit
            Contract Address:     0x069b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0
            Calldata:             ReportContext { config_digest: 0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, epoch_and_round: 4912644_u64, extra_hash: 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d }, 1720973048_u64, 0x2030100000000000000000000000000000000000000000000000000000000, array![100000608_u128, 100006353_u128, 100006353_u128, 100013900_u128], 42526329341833089_u128, 64858983089533_u128, array![Signature { r: 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, s: 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, public_key: 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2 }, Signature { r: 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, s: 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, public_key: 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711 }]
            Result:               success
        Fee Transfer Invocation
          Entry Point Selector:   transfer
          Contract Address:       0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Calldata:               ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 1945769550285990_u256
          Result:                 success: true
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_invoke_transaction_trace_full() {
    runner(&["get", "tx-trace", INVOKE_TX_HASH, "--full", "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                     INVOKE
        Transaction Hash:         0x026476da48e56e5e7025543ad0bb9105df00ee08571c6d17c4207462ff7717c4
        Execute Invocation
          Call Type:              CALL
          Calldata:               [0x1, 0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0, 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, 0x13, 0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x2, 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2, 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711]
          Caller Address:         0x0
          Calls
            Call Type:            CALL
            Calldata:             [0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x2, 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2, 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711]
            Caller Address:       0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
            Calls:                []
            Class Hash:           0x2f6d77cb0bca422706a91858dff62975aef4b8214520aadb1f0b39c51f5fde
            Contract Address:     0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0
            Entry Point Selector: 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8
            Entry Point Type:     EXTERNAL
            Events
              Data:               [0x5f5f9d1, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x0]
              Keys:               [0x19e22f866f4c5aead2809bf160d2b29e921e335d899979732101c6f3c38ff81, 0x1610, 0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1]
              Order:              0
            Execution Resources
              L1 Gas:             18
              L2 Gas:             0
            Is Reverted:          false
            Messages:             []
            Result:               []
          Class Hash:             0x5431265f9d2416426da800a23ddd3fe33db8e2b9fe96dbc48588ac3ac70c091
          Contract Address:       0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
          Entry Point Selector:   0x15d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad
          Entry Point Type:       EXTERNAL
          Events:                 []
          Execution Resources
            L1 Gas:               18
            L2 Gas:               0
          Is Reverted:            false
          Messages:               []
          Result:                 [0x1, 0x0]
        Execution Resources
          L1 Data Gas:            576
          L1 Gas:                 30
          L2 Gas:                 0
        Fee Transfer Invocation
          Call Type:              CALL
          Calldata:               [0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0x6e9aac6dc0ca6, 0x0]
          Caller Address:         0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
          Calls:                  []
          Class Hash:             0x4ad3c1dc8413453db314497945b6903e1c766495a1e60492d44da9c2a986e4b
          Contract Address:       0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Entry Point Selector:   0x83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e
          Entry Point Type:       EXTERNAL
          Events
            Data:                 [0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1, 0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0x6e9aac6dc0ca6, 0x0]
            Keys:                 [0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9]
            Order:                0
          Execution Resources
            L1 Gas:               4
            L2 Gas:               0
          Is Reverted:            false
          Messages:               []
          Result:                 [0x1]
        Validate Invocation
          Call Type:              CALL
          Calldata:               [0x1, 0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0, 0xc844fd57777b0cd7e75c8ea68deec0adf964a6308da7a58de32364b7131cc8, 0x13, 0x4c141c4019d33c08b1d94a85d54c4e92c2a5b2c5a2889f12ab81ecf220752, 0x4af604, 0x4e8d5ddbe29135d13a7edab22018a8f61cde628b468abfe63597996402ef4d, 0x6693f6f8, 0x2030100000000000000000000000000000000000000000000000000000000, 0x4, 0x5f5e360, 0x5f5f9d1, 0x5f5f9d1, 0x5f6174c, 0x971578a2056f81, 0x3afd28a5c57d, 0x2, 0x2f1f486eae02cc20f7fb08be2f88cbd44deca49bdc0ef031a4edf575f1cabf7, 0x2a539e3eaeadfc5ca897d7cbf38397fd7c6c6f3ccbc4f260d9d23a1e6bc5287, 0x6b72f55e68f1936feea371fab12ec039e8e1173579c03ac5c884ebf1653cce2, 0x4be40b6cab1d8e3acf5a99261cc3e07794b250a8afa04a1f44e2ac926895e6e, 0x167e81f05b6eddbe87180e9c27ba2f684a5d57709bb34c9e06f7d7f82b5c4b7, 0x319f9f7ce6e6c72e179557e79c57a542508867fa05b505cc32d2380ac72e711]
          Caller Address:         0x0
          Calls:                  []
          Class Hash:             0x5431265f9d2416426da800a23ddd3fe33db8e2b9fe96dbc48588ac3ac70c091
          Contract Address:       0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1
          Entry Point Selector:   0x162da33a4585851fe8d3af3c2a9c60b557814e221e0d4f30ff0b2189d9c7775
          Entry Point Type:       EXTERNAL
          Events:                 []
          Execution Resources
            L1 Gas:               8
            L2 Gas:               0
          Is Reverted:            false
          Messages:               []
          Result:                 [0x56414c4944]
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_declare_transaction_trace() {
    runner(&["get", "tx-trace", DECLARE_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   DECLARE
        Transaction Hash:       0x06054540622d534ffffb162a0e80c21bc106581eafeb3efad29385b78e04983d
        Validate Invocation
          Entry Point Selector: __validate_declare__
          Contract Address:     0x06aac79bb6c90e1e41c33cd20c67c0281c4a95f01b4e15ad0c3b53fcc6010cf8
          Calldata:             0x58e7b465e6c7651fa8697964a2d1ed93a62c0c00ba9876ddc4480dd0578d343
          Result:               success: 0x56414c4944
        Fee Transfer Invocation
          Entry Point Selector: transfer
          Contract Address:     0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Calldata:             ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 10343418238809876_u256
          Result:               success: true
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_declare_transaction_trace_full() {
    runner(&["get", "tx-trace", DECLARE_TX_HASH, "--full", "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   DECLARE
        Transaction Hash:       0x06054540622d534ffffb162a0e80c21bc106581eafeb3efad29385b78e04983d
        Execution Resources
          L1 Data Gas:          192
          L1 Gas:               1071
          L2 Gas:               0
        Fee Transfer Invocation
          Call Type:            CALL
          Calldata:             [0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0x24bf48b9e33714, 0x0]
          Caller Address:       0x6aac79bb6c90e1e41c33cd20c67c0281c4a95f01b4e15ad0c3b53fcc6010cf8
          Calls:                []
          Class Hash:           0x4ad3c1dc8413453db314497945b6903e1c766495a1e60492d44da9c2a986e4b
          Contract Address:     0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Entry Point Selector: 0x83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e
          Entry Point Type:     EXTERNAL
          Events
            Data:               [0x6aac79bb6c90e1e41c33cd20c67c0281c4a95f01b4e15ad0c3b53fcc6010cf8, 0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0x24bf48b9e33714, 0x0]
            Keys:               [0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9]
            Order:              0
          Execution Resources
            L1 Gas:             4
            L2 Gas:             0
          Is Reverted:          false
          Messages:             []
          Result:               [0x1]
        Validate Invocation
          Call Type:            CALL
          Calldata:             [0x58e7b465e6c7651fa8697964a2d1ed93a62c0c00ba9876ddc4480dd0578d343]
          Caller Address:       0x0
          Calls:                []
          Class Hash:           0x450f568a8cb6ea1bcce446355e8a1c2e5852a6b8dc3536f495cdceb62e8a7e2
          Contract Address:     0x6aac79bb6c90e1e41c33cd20c67c0281c4a95f01b4e15ad0c3b53fcc6010cf8
          Entry Point Selector: 0x289da278a8dc833409cabfdad1581e8e7d40e42dcaed693fa4008dcdb4963b3
          Entry Point Type:     EXTERNAL
          Events:               []
          Execution Resources
            L1 Gas:             8
            L2 Gas:             0
          Is Reverted:          false
          Messages:             []
          Result:               [0x56414c4944]
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_deploy_account_transaction_trace() {
    runner(&["get", "tx-trace", DEPLOY_ACCOUNT_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   DEPLOY_ACCOUNT
        Transaction Hash:       0x06718b783a0b888f5421c4eb76a532feb9fd5167b2b09274298f79798c782b32
        Validate Invocation
          Entry Point Selector: __validate_deploy__
          Contract Address:     0x0563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677
          Calldata:             0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f, 0x23141, 0x796b0283375b9aa6fc0ac6b9ea1f98584f8464a66f05f29c3deb4c5eeea5263
          Result:               success: 0x56414c4944
        Constructor Invocation
          Entry Point Selector: constructor
          Contract Address:     0x0563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677
          Calldata:             0x796b0283375b9aa6fc0ac6b9ea1f98584f8464a66f05f29c3deb4c5eeea5263
          Result:               success
        Fee Transfer Invocation
          Entry Point Selector: transfer
          Contract Address:     0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Calldata:             ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 51588769029944096_u256
          Result:               success: true
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_deploy_account_transaction_trace_full() {
    runner(&["get", "tx-trace", DEPLOY_ACCOUNT_TX_HASH, "--full", "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   DEPLOY_ACCOUNT
        Transaction Hash:       0x06718b783a0b888f5421c4eb76a532feb9fd5167b2b09274298f79798c782b32
        Constructor Invocation
          Call Type:            CALL
          Calldata:             [0x796b0283375b9aa6fc0ac6b9ea1f98584f8464a66f05f29c3deb4c5eeea5263]
          Caller Address:       0x0
          Calls:                []
          Class Hash:           0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f
          Contract Address:     0x563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677
          Entry Point Selector: 0x28ffe4ff0f226a9107253e17a904099aa4f63a02a5621de0576e5aa71bc5194
          Entry Point Type:     CONSTRUCTOR
          Events
            Data:               [0x796b0283375b9aa6fc0ac6b9ea1f98584f8464a66f05f29c3deb4c5eeea5263]
            Keys:               [0x38f6a5b87c23cee6e7294bcc3302e95019f70f81586ff3cac38581f5ca96381]
            Order:              0
          Execution Resources
            L1 Gas:             0
            L2 Gas:             0
          Is Reverted:          false
          Messages:             []
          Result:               []
        Execution Resources
          L1 Data Gas:          0
          L1 Gas:               0
          L2 Gas:               377050
        Fee Transfer Invocation
          Call Type:            CALL
          Calldata:             [0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0xb747b64c6df320, 0x0]
          Caller Address:       0x563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677
          Calls:                []
          Class Hash:           0x5327164fa21dca89a92e8eae8a5b7ab90f58373e71f0a16d285e5a4abe5a3cf
          Contract Address:     0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Entry Point Selector: 0x83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e
          Entry Point Type:     EXTERNAL
          Events
            Data:               [0x563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677, 0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8, 0xb747b64c6df320, 0x0]
            Keys:               [0x99cd8bde557814842a3121e8ddfd433a539b8c9f14bf31ebf108d12e6196e9]
            Order:              0
          Execution Resources
            L1 Gas:             0
            L2 Gas:             0
          Is Reverted:          false
          Messages:             []
          Result:               [0x1]
        Validate Invocation
          Call Type:            CALL
          Calldata:             [0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f, 0x23141, 0x796b0283375b9aa6fc0ac6b9ea1f98584f8464a66f05f29c3deb4c5eeea5263]
          Caller Address:       0x0
          Calls:                []
          Class Hash:           0x61dac032f228abef9c6626f995015233097ae253a7f72d68552db02f2971b8f
          Contract Address:     0x563870107a0a2c8cf34d2a42118dc52706a7eae7c1c741d32abec98d3238677
          Entry Point Selector: 0x36fcbf06cd96843058359e1a75928beacfac10727dab22a3972f0af8aa92895
          Entry Point Type:     EXTERNAL
          Events:               []
          Execution Resources
            L1 Gas:             0
            L2 Gas:             0
          Is Reverted:          false
          Messages:             []
          Result:               [0x56414c4944]
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_l1_handler_transaction_trace() {
    runner(&["get", "tx-trace", L1_HANDLER_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                     L1_HANDLER
        Transaction Hash:         0x004c8c57b3ab646ef56aef3def69a01bc86d049b98f25ebfe3699334d86c24d5
        Function Invocation
          Entry Point Selector:   receive_commitment
          Contract Address:       0x0763c1a0ec1d64afe2d8d0a2c0cab6fd494dcb26d08ef1020b27aa5695761e21
          Calldata:               0x423f7744017600727ce4789933e4648068835e28, 99359224995532825384289367229767519998085228603184343483478058520072918113310_u256, 6289449_u256
          Result:                 success
          Calls
            Entry Point Selector: receive_hash
            Contract Address:     0x020f6d32589a0d57c72faed530354bac49144ca99aff3429cc3284514583b595
            Calldata:             99359224995532825384289367229767519998085228603184343483478058520072918113310_u256, 6289449_u256
            Result:               success
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_l1_handler_transaction_trace_full() {
    runner(&["get", "tx-trace", L1_HANDLER_TX_HASH, "--full", "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                     L1_HANDLER
        Transaction Hash:         0x004c8c57b3ab646ef56aef3def69a01bc86d049b98f25ebfe3699334d86c24d5
        Execution Resources
          L1 Data Gas:            192
          L1 Gas:                 18783
          L2 Gas:                 401920
        Function Invocation
          Call Type:              CALL
          Calldata:               [0x423f7744017600727ce4789933e4648068835e28, 0x43c585509832d6c8e71cc49e0a1b9c1e, 0xdbab5414cf45b42ceac0b391fb86d03b, 0x5ff829, 0x0]
          Caller Address:         0x0
          Calls
            Call Type:            CALL
            Calldata:             [0x43c585509832d6c8e71cc49e0a1b9c1e, 0xdbab5414cf45b42ceac0b391fb86d03b, 0x5ff829, 0x0]
            Caller Address:       0x763c1a0ec1d64afe2d8d0a2c0cab6fd494dcb26d08ef1020b27aa5695761e21
            Calls:                []
            Class Hash:           0x48d1f93626722872832416241a30c20bb77403b48249e65bebae67ab7a5329
            Contract Address:     0x20f6d32589a0d57c72faed530354bac49144ca99aff3429cc3284514583b595
            Entry Point Selector: 0xbef29f825020c5aac1121de2686d2010bd562ce4612350d4668e7812b998d7
            Entry Point Type:     EXTERNAL
            Events
              Data:               [0x5ff829, 0x0, 0x43c585509832d6c8e71cc49e0a1b9c1e, 0xdbab5414cf45b42ceac0b391fb86d03b]
              Keys:               [0x3ea6420d00c650e7e902fc0797c87dc37be11f59689cd106c2948c1d91f7b60]
              Order:              0
            Execution Resources
              L1 Gas:             2
              L2 Gas:             0
            Is Reverted:          false
            Messages:             []
            Result:               []
          Class Hash:             0x18c9ce7ffa88f15bd1fcda1350cb66cc5c369bc924e5dc108be1c9317298c99
          Contract Address:       0x763c1a0ec1d64afe2d8d0a2c0cab6fd494dcb26d08ef1020b27aa5695761e21
          Entry Point Selector:   0x3fa70707d0e831418fb142ca8fb7483611b84e89c0c42bf1fc2a7a5c40890ad
          Entry Point Type:       L1_HANDLER
          Events
            Data:                 [0x43c585509832d6c8e71cc49e0a1b9c1e, 0xdbab5414cf45b42ceac0b391fb86d03b, 0x5ff829, 0x0]
            Keys:                 [0xe1eadb452a63ae1892154f372496b89e12ce9e7ce6fa424ec2378abd0b7fa]
            Order:                1
          Execution Resources
            L1 Gas:               6
            L2 Gas:               0
          Is Reverted:            false
          Messages:               []
          Result:                 []
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_reverted_invoke_transaction_trace() {
    runner(&["get", "tx-trace", REVERTED_INVOKE_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   INVOKE
        Transaction Hash:       0x00fecca6a328dd11f40b79c30fe22d23bc6975d1a0923a95b90aff4016a84333
        Validate Invocation
          Entry Point Selector: __validate__
          Contract Address:     0x04c1d9da136846ab084ae18cf6ce7a652df7793b666a16ce46b1bf5850cc739d
          Calldata:             array![Call { to: ContractAddress(0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a), selector: 0x3d0bcca55c118f88a08e0fcc06f43906c0c174feb52ebc83f0fa28a1f59ed67, calldata: array![0x69, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x5ba38b5e260, 0x4254432f555344, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x4f097b6c60, 0x4554482f555344, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0xf40e2, 0x555344432f555344, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x5bcd6e7b560, 0x574254432f555344, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x5f8dab7, 0x574254432f425443, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x5420a5011c0, 0x4254432f455552, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x30117350, 0x554e492f555344, 0x0, 0x0, 0x669620b5, 0x42494e414e4345, 0x505241474d41, 0x381fad0, 0x5354524b2f555344, 0x0, 0x0, 0x66961f4f, 0x444546494c4c414d41, 0x505241474d41, 0x5bd58fe0400, 0x4254432f555344, 0x0, 0x0, 0x66961f80, 0x444546494c4c414d41, 0x505241474d41, 0x4f5b3ada40, 0x4554482f555344, 0x0, 0x0, 0x66961f65, 0x444546494c4c414d41, 0x505241474d41, 0x5f40d08, 0x4441492f555344, 0x0, 0x0, 0x66961f54, 0x444546494c4c414d41, 0x505241474d41, 0xf3e26, 0x555344432f555344, 0x0, 0x0, 0x66961f64, 0x444546494c4c414d41, 0x505241474d41, 0xf3f7d, 0x555344542f555344, 0x0, 0x0, 0x66961f74, 0x444546494c4c414d41, 0x505241474d41, 0x5c118a08500, 0x574254432f555344, 0x0, 0x0, 0x66961f74, 0x444546494c4c414d41, 0x505241474d41, 0x5f9c58e, 0x574254432f425443, 0x0, 0x0, 0x66961f89, 0x444546494c4c414d41, 0x505241474d41, 0x5d2616ffc0, 0x5753544554482f555344, 0x0, 0x0, 0x66961ebc, 0x444546494c4c414d41, 0x505241474d41, 0x73c7d8, 0x4c4f5244532f555344, 0x0, 0x0, 0x66961f34, 0x444546494c4c414d41, 0x505241474d41, 0x5ef4278, 0x4c5553442f555344, 0x0, 0x0, 0x66961f4b, 0x444546494c4c414d41, 0x505241474d41, 0x30385c40, 0x554e492f555344, 0x0, 0x0, 0x66961f4b, 0x444546494c4c414d41, 0x505241474d41, 0x384aa50, 0x5354524b2f555344, 0x0, 0x0, 0x66961f64, 0x444546494c4c414d41, 0x505241474d41, 0x1848e50, 0x5a454e442f555344, 0x0, 0x0, 0x66961f30, 0x444546494c4c414d41, 0x505241474d41, 0x23dd05cc0, 0x4450492f555344, 0x0, 0x0, 0x66961f1f, 0x444546494c4c414d41, 0x505241474d41, 0x673a2c, 0x4e5354522f555344, 0x0, 0x0, 0x66961f77, 0x444546494c4c414d41, 0x505241474d41, 0x4f5e080400, 0x53544554482f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x5c0eaabfe40, 0x4254432f555344, 0x224b4480d2bdb64000, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x4f3c4b8a80, 0x4554482f555344, 0x1d847d1fe713a80000, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x5c0eaabfe40, 0x574254432f555344, 0x224b4480d2bdb64000, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x5cf442a6c0, 0x5753544554482f555344, 0x22a0cc3b9ea4ec000, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x7423cd, 0x4c4f5244532f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x5f5d37e, 0x4c5553442f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x302ce9ad, 0x554e492f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x385ece5, 0x5354524b2f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x23dc11a80, 0x4450492f555344, 0x0, 0x0, 0x669620b4, 0x4745434b4f5445524d494e414c, 0x505241474d41, 0x66069a, 0x4e5354522f555344, 0x0, 0x0, 0x669620b0, 0x4b55434f494e, 0x505241474d41, 0x5ba37e7e400, 0x4254432f555344, 0x0, 0x0, 0x669620b1, 0x4b55434f494e, 0x505241474d41, 0x4f0678dac0, 0x4554482f555344, 0x0, 0x0, 0x6696208c, 0x4b55434f494e, 0x505241474d41, 0xf3fe8, 0x555344432f555344, 0x0, 0x0, 0x669603da, 0x4b55434f494e, 0x505241474d41, 0x5d40b224180, 0x574254432f555344, 0x0, 0x0, 0x66961db4, 0x4b55434f494e, 0x505241474d41, 0x5f479a0, 0x574254432f425443, 0x0, 0x0, 0x6696196d, 0x4b55434f494e, 0x505241474d41, 0x541528d3900, 0x4254432f455552, 0x0, 0x0, 0x66962091, 0x4b55434f494e, 0x505241474d41, 0x301c21b0, 0x554e492f555344, 0x0, 0x0, 0x66962068, 0x4b55434f494e, 0x505241474d41, 0x3836230, 0x5354524b2f555344, 0x0, 0x0, 0x66962095, 0x4b55434f494e, 0x505241474d41, 0x1831d8f, 0x5a454e442f555344, 0x0, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x5b9e154b130, 0x4254432f555344, 0x2430f365f033853c5a0df5000, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x4f01d00e6b, 0x4554482f555344, 0x42e5e9e9f1a59e453032300, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x5f30585, 0x4441492f555344, 0x2e4039817dea1661600, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0xf3f9f, 0x555344432f555344, 0x3b428109c8034dc0, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x5f690c7, 0x574254432f425443, 0x0, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x3012ac5f, 0x554e492f555344, 0x1073e837bea792fb97c00, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x382774d, 0x5354524b2f555344, 0x3c61f2487c0437dd00, 0x0, 0x669620b4, 0x48554f4249, 0x505241474d41, 0x4e76c19c49, 0x53544554482f555344, 0xfc9be485169415bb3600, 0x0, 0x669620a9, 0x4f4b58, 0x505241474d41, 0xf3f6d, 0x555344432f555344, 0x594af543ee31580, 0x0, 0x669620b4, 0x4f4b58, 0x505241474d41, 0x3817973, 0x5354524b2f555344, 0x40038c75e613d8000000, 0x0, 0x669620b0, 0x4249545354414d50, 0x505241474d41, 0x5bafefc3f00, 0x4254432f555344, 0x0, 0x0, 0x669620b2, 0x4249545354414d50, 0x505241474d41, 0x4f141f1e00, 0x4554482f555344, 0x0, 0x0, 0x669620b3, 0x4249545354414d50, 0x505241474d41, 0x5f5ae38, 0x4441492f555344, 0x0, 0x0, 0x669620b2, 0x4249545354414d50, 0x505241474d41, 0xf4236, 0x555344432f555344, 0x0, 0x0, 0x669620b0, 0x4249545354414d50, 0x505241474d41, 0xf4361, 0x555344542f555344, 0x0, 0x0, 0x669620b2, 0x4249545354414d50, 0x505241474d41, 0x5fcbed0, 0x574254432f425443, 0x0, 0x0, 0x669620b1, 0x4249545354414d50, 0x505241474d41, 0x541dab04c00, 0x4254432f455552, 0x0, 0x0, 0x669620b1, 0x4249545354414d50, 0x505241474d41, 0x306fe700, 0x554e492f555344, 0x0, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x5ba38b5e260, 0x4254432f555344, 0x27c569d4, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x4f097b6c60, 0x4554482f555344, 0xafdd6bb, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x5f494f8, 0x4441492f555344, 0x1f1a1, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0xf40e2, 0x555344432f555344, 0xbd615e, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x5c158e91fa0, 0x574254432f555344, 0x9def, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x300fecb0, 0x554e492f555344, 0x31c23, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x381fad0, 0x5354524b2f555344, 0x4e138, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x684b37, 0x4e5354522f555344, 0xd120, 0x0, 0x669620b5, 0x4d455843, 0x505241474d41, 0x4f0d6a80df, 0x53544554482f555344, 0x7927, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x5ba47b37840, 0x4254432f555344, 0x1e2c0445, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x4f06fa8de0, 0x4554482f555344, 0x1b2fe21e, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x5f30e58, 0x4441492f555344, 0x5234, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0xf40e2, 0x555344432f555344, 0x2eb5e6, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x5bcbba2d000, 0x574254432f555344, 0x3edd, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x5f72d07, 0x574254432f425443, 0x0, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x3009d230, 0x554e492f555344, 0x403c5e, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x3813780, 0x5354524b2f555344, 0x3bf098, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x67263f, 0x4e5354522f555344, 0x49dc8, 0x0, 0x669620b5, 0x47415445494f, 0x505241474d41, 0x4eb9d20d40, 0x53544554482f555344, 0x7709, 0x0, 0x669620b4, 0x535441524b4e4554, 0x505241474d41, 0x739dd0, 0x4c4f5244532f555344, 0x0, 0x0, 0x669620b4, 0x535441524b4e4554, 0x505241474d41, 0x38ca0de, 0x5354524b2f555344, 0x0, 0x0, 0x669620b4, 0x535441524b4e4554, 0x505241474d41, 0x145ebb8, 0x5a454e442f555344, 0x0, 0x0, 0x669620b4, 0x535441524b4e4554, 0x505241474d41, 0x66f7e4, 0x4e5354522f555344, 0x0, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x5ba4440a27f, 0x4254432f555344, 0x1dfb41680274020, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x4f06b2ff81, 0x4554482f555344, 0xe55e836a1606d8, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x5f43df3, 0x4441492f555344, 0x381192df62ca, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0xf4003, 0x555344432f555344, 0x980ea8bc6afe, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x5ba7382281f, 0x574254432f555344, 0xb923b3f3c90, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x5f5f487, 0x574254432f425443, 0xb5625bc6, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x5424397c400, 0x4254432f455552, 0x268ad4bed860, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x300d0762, 0x554e492f555344, 0x10bd965e201d2, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x381c78f, 0x5354524b2f555344, 0x241a692d1bebb, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x18034f8, 0x5a454e442f555344, 0xd8aa8ff2692, 0x0, 0x669620b4, 0x4259424954, 0x505241474d41, 0x4f0be9d82d, 0x53544554482f555344, 0xe7187793b8ece, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x5b9bda49300, 0x4254432f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x4f0c1cd814, 0x4554482f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x5f2c163, 0x4441492f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0xf3538, 0x555344542f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x5b9bda49300, 0x574254432f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x5cb548ee34, 0x5753544554482f555344, 0x0, 0x0, 0x669620b4, 0x50524f50454c4c4552, 0x505241474d41, 0x23d78df7c, 0x4450492f555344, 0x0, 0x0, 0x669620b5, 0x494e444558434f4f50, 0x505241474d41, 0x23ca8cf46, 0x4450492f555344, 0x329efd877fa, 0x0, 0x669620b8, 0x42494e414e4345, 0x505241474d41, 0x23cec8a5c, 0x4450492f555344, 0x0, 0x0, 0x669620b4, 0x444546494c4c414d41, 0x505241474d41, 0x23e33cd4b, 0x4450492f555344, 0x0] }]
          Result:               success: 0x56414c4944
        Execute Invocation
          Revert Reason:        Transaction execution has failed:
                                0: Error in the called contract (contract address: 0x04c1d9da136846ab084ae18cf6ce7a652df7793b666a16ce46b1bf5850cc739d, class hash: 0x01a736d6ed154502257f02b1ccdf4d9d1089f80811cd6acad48e6b6a9d1f2003, selector: 0x015d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad):
                                Error at pc=0:15647:
                                Cairo traceback (most recent call last):
                                Unknown location (pc=0:233)
                                Unknown location (pc=0:5191)
                                Unknown location (pc=0:11307)

                                1: Error in the called contract (contract address: 0x036031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a, class hash: 0x05e269051bec902aa2bd421d348e023c3893c4ff93de6c5f4b8964cd67cc3fc5, selector: 0x03d0bcca55c118f88a08e0fcc06f43906c0c174feb52ebc83f0fa28a1f59ed67):
                                Execution failed. Failure reason:
                                Error in contract (contract address: 0x036031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a, class hash: 0x05e269051bec902aa2bd421d348e023c3893c4ff93de6c5f4b8964cd67cc3fc5, selector: 0x03d0bcca55c118f88a08e0fcc06f43906c0c174feb52ebc83f0fa28a1f59ed67):
                                0x4578697374696e6720656e747279206973206d6f726520726563656e74 ('Existing entry is more recent').

        Fee Transfer Invocation
          Entry Point Selector: transfer
          Contract Address:     0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
          Calldata:             ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 9402441379216_u256
          Result:               success: true
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_invoke_transaction_trace_json() {
    let output = runner(&["--json", "get", "tx-trace", INVOKE_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stderr_eq("");

    let json: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let trace = &json["transaction_trace"];

    assert_eq!(json["command"], "get tx-trace");
    assert_eq!(json["type"], "response");
    assert_eq!(trace["type"], "INVOKE");

    assert_eq!(
        trace["validate_invocation"]["contract_address"],
        "0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1"
    );
    assert_eq!(
        trace["validate_invocation"]["entry_point_selector"],
        "0x162da33a4585851fe8d3af3c2a9c60b557814e221e0d4f30ff0b2189d9c7775"
    );
    assert_eq!(trace["validate_invocation"]["result"][0], "0x56414c4944");

    assert_eq!(
        trace["execute_invocation"]["contract_address"],
        "0x350461cc881640ebbcebf747107d456ef008ec455cd95d2b76a7d9face671f1"
    );
    assert_eq!(
        trace["execute_invocation"]["calls"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        trace["execute_invocation"]["calls"][0]["contract_address"],
        "0x69b1360564534bf59fa889041a60f2c60ef5b259cfbf87a436867538e2c53e0"
    );
    assert_eq!(
        trace["execute_invocation"]["calls"][0]["events"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    assert_eq!(trace["execution_resources"]["l1_gas"], 30);
    assert_eq!(trace["execution_resources"]["l1_data_gas"], 576);
    assert_eq!(trace["execution_resources"]["l2_gas"], 0);
    assert_eq!(
        trace["fee_transfer_invocation"]["contract_address"],
        "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"
    );
}
#[tokio::test]
async fn test_transaction_trace_alias() {
    runner(&["get", "transaction-trace", DECLARE_TX_HASH, "--url", URL])
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Transaction trace retrieved

        Type:                   DECLARE
        Transaction Hash:       0x06054540622d534ffffb162a0e80c21bc106581eafeb3efad29385b78e04983d
        Validate Invocation
          Entry Point Selector: __validate_declare__
          Contract Address:     0x06aac79bb6c90e1e41c33cd20c67c0281c4a95f01b4e15ad0c3b53fcc6010cf8
          Calldata:             0x58e7b465e6c7651fa8697964a2d1ed93a62c0c00ba9876ddc4480dd0578d343
          Result:               success: 0x56414c4944
        Fee Transfer Invocation
          Entry Point Selector: transfer
          Contract Address:     0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d
          Calldata:             ContractAddress(0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8), 10343418238809876_u256
          Result:               success: true
    "})
        .stderr_eq("");
}

#[tokio::test]
async fn test_human_trace_falls_back_when_class_is_unavailable() {
    let mock_server = MockServer::start().await;
    let mut nested_invocation = invocation();
    nested_invocation["contract_address"] = json!("0x789");
    let mut trace = trace();
    trace["execute_invocation"]["calls"] = json!([nested_invocation]);
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": trace
        })),
    )
    .await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({ "method": "starknet_getClass" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": 28, "message": "Class hash not found" }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    runner(&[
        "get",
        "tx-trace",
        TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ])
    .assert()
    .success()
    .stdout_eq(indoc! {r"
            [WARNING] Could not fetch contract classes needed to decode the trace:
            - class hash: 0x456, contract addresses: 0x123, 0x789 — Class hash not found
            Affected calls are displayed as raw felts.

            Success: Transaction trace retrieved

            Type:                     INVOKE
            Transaction Hash:         0x0000000000000000000000000000000000000000000000000000000000000abc
            Execute Invocation
              Entry Point Selector:   0x240060cdb34fcc260f41eac7474ee1d7c80b7e3607daff9ac67c7ea2ebb1c44
              Contract Address:       0x0000000000000000000000000000000000000000000000000000000000000123
              Calldata:               0x7
              Result:                 success: 0x9
              Calls
                Entry Point Selector: 0x240060cdb34fcc260f41eac7474ee1d7c80b7e3607daff9ac67c7ea2ebb1c44
                Contract Address:     0x0000000000000000000000000000000000000000000000000000000000000789
                Calldata:             0x7
                Result:               success: 0x9
        "})
    .stderr_eq("");
}

#[tokio::test]
async fn test_human_trace_warns_when_fetched_abi_cannot_decode_trace() {
    let mock_server = MockServer::start().await;
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": trace()
        })),
    )
    .await;
    mock_contract_class(&mock_server, json!([])).await;

    runner(&[
        "get",
        "tx-trace",
        TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ])
    .assert()
    .success()
    .stdout_eq(indoc! {r"
            [WARNING] Some trace data could not be decoded with the fetched ABIs. Raw felts are shown instead.

            Success: Transaction trace retrieved

            Type:                   INVOKE
            Transaction Hash:       0x0000000000000000000000000000000000000000000000000000000000000abc
            Execute Invocation
              Entry Point Selector: 0x240060cdb34fcc260f41eac7474ee1d7c80b7e3607daff9ac67c7ea2ebb1c44
              Contract Address:     0x0000000000000000000000000000000000000000000000000000000000000123
              Calldata:             0x7
              Result:               success: 0x9
        "})
    .stderr_eq("");
}

#[tokio::test]
async fn test_transaction_not_found() {
    runner(&["get", "tx-trace", TRANSACTION_HASH, "--url", URL])
        .assert()
        .failure()
        .stdout_eq("")
        .stderr_eq(indoc! {r"
            Command: get tx-trace
            Error: Transaction with provided hash was not found (does not exist)
        "});
}

#[tokio::test]
async fn test_trace_not_available() {
    let mock_server = MockServer::start().await;
    mock_trace(
        &mock_server,
        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": 10,
                "message": "No trace available for transaction",
                "data": { "status": "RECEIVED" }
            }
        })),
    )
    .await;

    runner(&[
        "get",
        "tx-trace",
        TRANSACTION_HASH,
        "--url",
        &mock_server.uri(),
    ])
    .assert()
    .failure()
    .stdout_eq("")
    .stderr_eq(indoc! {r"
            Command: get tx-trace
            Error: No trace is available for the transaction (status: Received)
        "});
}
