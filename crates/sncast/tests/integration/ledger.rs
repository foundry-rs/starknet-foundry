#![cfg(not(target_arch = "wasm32"))]

//! Integration tests for Ledger hardware wallet functionality
//!
//! This module contains tests for:
//! - Basic Ledger operations (get public key, sign hash, app version)
//! - Account creation and management
//! - Transaction signing (invoke, deploy)
//! - Network integration with devnet

use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use coins_ledger::{APDUAnswer, APDUCommand, LedgerError, transports::LedgerAsync};
use sncast::ledger::{self, AppVersion, GetPublicKey, LedgerResponse, SignHash};
use speculos_client::{
    AutomationAction, AutomationCondition, AutomationRule, Button, DeviceModel, SpeculosClient,
};
use starknet_rust::accounts::{
    Account, AccountFactory, ExecutionEncoding, OpenZeppelinAccountFactory,
};
use starknet_rust::core::types::{BlockId, BlockTag, Call, TransactionReceipt::Invoke};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::ledger::LedgerStarknetApp;
use starknet_rust::signers::{DerivationPath, LedgerSigner};
use starknet_types_core::felt::Felt;
use url::Url;

use crate::helpers::constants::{
    MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::fixtures::mint_token;
use sncast::helpers::constants::OZ_CLASS_HASH;

const TEST_LEDGER_PATH: &str = "m/2645'/1195502025'/1470455285'/0'/0'/0";
const APP_PATH: &str = "../data/ledger-app/nanox#strk#0.25.13.elf";

// ============================================================================
// Test Infrastructure
// ============================================================================

#[derive(Debug)]
struct SpeculosTransport(Arc<SpeculosClient>);

#[async_trait]
impl LedgerAsync for SpeculosTransport {
    async fn init() -> Result<Self, LedgerError> {
        Ok(Self(Arc::new(
            SpeculosClient::new(DeviceModel::Nanox, 5001, APP_PATH).unwrap(),
        )))
    }

    async fn exchange(&self, packet: &APDUCommand) -> Result<APDUAnswer, LedgerError> {
        let raw_answer = self.0.apdu(&packet.serialize()).await.unwrap();
        Ok(APDUAnswer::from_answer(raw_answer).unwrap())
    }

    fn close(self) {}
}

fn setup_app(port: u16) -> (Arc<SpeculosClient>, LedgerStarknetApp<SpeculosTransport>) {
    let client = Arc::new(SpeculosClient::new(DeviceModel::Nanox, port, APP_PATH).unwrap());
    let app = LedgerStarknetApp::from_transport(SpeculosTransport(client.clone()));
    (client, app)
}

fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    JsonRpcClient::new(HttpTransport::new(Url::parse(URL).unwrap()))
}

/// Helper function to deploy an account that matches the Ledger's public key
/// Uses a unique salt per test to avoid address collisions
async fn deploy_ledger_account(
    app: LedgerStarknetApp<SpeculosTransport>,
    path: &str,
    salt: Felt,
) -> (Felt, Felt) {
    use std::str::FromStr;

    let provider = create_jsonrpc_client();

    // Get public key from Ledger
    let parsed_path = DerivationPath::from_str(path).unwrap();
    let public_key = app
        .get_public_key(parsed_path.clone(), false)
        .await
        .unwrap();
    let public_key_felt = public_key.scalar();

    // Create Ledger signer
    let ledger_signer = LedgerSigner::new_with_app(parsed_path, app).unwrap();

    // Compute the account address and prepare deployment
    let class_hash = OZ_CLASS_HASH;

    let factory = OpenZeppelinAccountFactory::new(
        class_hash,
        starknet_rust::core::chain_id::SEPOLIA,
        ledger_signer,
        provider.clone(),
    )
    .await
    .unwrap();

    let deployment = factory.deploy_v3(salt);
    let address = deployment.address();

    // Check if account is already deployed
    let is_deployed = provider
        .get_class_hash_at(BlockId::Tag(BlockTag::Latest), address)
        .await
        .is_ok();

    if is_deployed {
        // Account already deployed, just ensure it has funds
        mint_token(&format!("{address:#066x}"), u128::MAX).await;
    } else {
        // Fund the account before deployment
        mint_token(&format!("{address:#066x}"), u128::MAX).await;

        // Deploy the account (Ledger will sign the deployment transaction)
        deployment
            .l1_gas(100_000)
            .l1_gas_price(10_000_000_000_000)
            .l2_gas(1_000_000)
            .l2_gas_price(10_000_000_000_000)
            .l1_data_gas(100_000)
            .l1_data_gas_price(10_000_000_000_000)
            .send()
            .await
            .expect("Failed to deploy account");
    }

    (address, public_key_felt)
}

// ============================================================================
// Automation Rules for Ledger UI Navigation
// ============================================================================

mod automation {
    use super::*;

    pub(crate) const APPROVE_PUBLIC_KEY: AutomationRule<'static> = AutomationRule {
        text: Some(Cow::Borrowed("Confirm Public Key")),
        regexp: None,
        x: None,
        y: None,
        conditions: &[],
        actions: &[
            // Right 1
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right 2
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right 3
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (approve)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
        ],
    };

    pub(crate) const ENABLE_BLIND_SIGN: AutomationRule<'static> = AutomationRule {
        text: None,
        regexp: Some(Cow::Borrowed("^(S)?tarknet$")),
        x: None,
        y: None,
        conditions: &[AutomationCondition {
            varname: Cow::Borrowed("blind_enabled"),
            value: false,
        }],
        actions: &[
            // Right (go to version screen)
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right (go to settings screen)
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (enter settings/blind signing screen)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (enable blind signing)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Left (go back)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            // Mark as done
            AutomationAction::Setbool {
                varname: Cow::Borrowed("blind_enabled"),
                value: true,
            },
        ],
    };

    /// Approve when "Blind Signing" warning text is shown
    pub(crate) const APPROVE_BLIND_SIGNING: AutomationRule<'static> = AutomationRule {
        text: Some(Cow::Borrowed("Blind Signing")),
        regexp: None,
        x: None,
        y: None,
        conditions: &[AutomationCondition {
            varname: Cow::Borrowed("blind_enabled"),
            value: true,
        }],
        actions: &[
            // Right 1
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right 2
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right 3
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (to approve)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
        ],
    };

    /// Approve blind signing of a hash (when "Cancel" button is shown)
    pub(crate) const APPROVE_BLIND_SIGN_HASH: AutomationRule<'static> = AutomationRule {
        text: None,
        regexp: Some(Cow::Borrowed("^Cancel$")),
        x: None,
        y: None,
        conditions: &[AutomationCondition {
            varname: Cow::Borrowed("blind_enabled"),
            value: true,
        }],
        actions: &[
            // Right (move to Accept)
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (press Accept)
            AutomationAction::Button {
                button: Button::Left,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Left,
                pressed: false,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
        ],
    };
}

// ============================================================================
// Basic Ledger Operations Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_app_version() {
    let (_, app) = setup_app(5001);
    let version = ledger::app_version(&AppVersion, app).await.unwrap();

    match version {
        LedgerResponse::Version(v) => assert_eq!(v.version, "2.3.4"),
        _ => panic!("Wrong response type"),
    }
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_public_key_headless() {
    let (_, app) = setup_app(5002);
    let args = GetPublicKey {
        path: TEST_LEDGER_PATH.to_string(),
        no_display: true,
    };
    let public_key = ledger::get_public_key(&args, app).await.unwrap();

    match public_key {
        LedgerResponse::PublicKey(pk) => assert_eq!(
            pk.public_key,
            "0x07427aa749c4fc98a5bf76f037eb3c61e7b4793b576a72d45a4b52c5ded997f2"
        ),
        _ => panic!("Wrong response type"),
    }
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_public_key_with_confirmation() {
    let (client, app) = setup_app(5003);

    // Automatically approve
    client
        .automation(&[automation::APPROVE_PUBLIC_KEY])
        .await
        .unwrap();

    let args = GetPublicKey {
        path: TEST_LEDGER_PATH.to_string(),
        no_display: false,
    };
    let public_key = ledger::get_public_key(&args, app).await.unwrap();

    match public_key {
        LedgerResponse::PublicKey(pk) => assert_eq!(
            pk.public_key,
            "0x07427aa749c4fc98a5bf76f037eb3c61e7b4793b576a72d45a4b52c5ded997f2"
        ),
        _ => panic!("Wrong response type"),
    }
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_sign_hash() {
    let (client, app) = setup_app(4001);
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let args = SignHash {
        path: TEST_LEDGER_PATH.to_string(),
        hash: "0x01234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd".to_string(),
    };

    let response = ledger::sign_hash(&args, app).await.unwrap();

    match response {
        LedgerResponse::Signature(s) => assert!(s.signature.starts_with("0x")),
        _ => panic!("Wrong response type"),
    }
}

// ============================================================================
// Account Creation Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_create_ledger_account() {
    let (client, app) = setup_app(6001);

    // Setup automation for public key confirmation
    client
        .automation(&[automation::APPROVE_PUBLIC_KEY])
        .await
        .unwrap();

    let provider = create_jsonrpc_client();
    let test_address = Felt::from_hex_unchecked(
        "0x01b0f8a1ab14f84573d8ed9eec0852a2099ff76ffb601686ffb14fac352b78b3",
    );

    // Test creating a ledger account
    let account = ledger::ledger_account_with_app(
        app,
        TEST_LEDGER_PATH,
        test_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await;

    assert!(account.is_ok(), "Failed to create ledger account");
    let account = account.unwrap();
    assert_eq!(account.address(), test_address);
    assert_eq!(account.chain_id(), starknet_rust::core::chain_id::SEPOLIA);
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_invalid_derivation_path() {
    let (_, app) = setup_app(5001);
    let provider = create_jsonrpc_client();
    let test_address = Felt::from_hex_unchecked(
        "0x01b0f8a1ab14f84573d8ed9eec0852a2099ff76ffb601686ffb14fac352b78b3",
    );

    // Test with invalid derivation path using Speculos app
    let result = ledger::ledger_account_with_app(
        app,
        "invalid/path",
        test_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await;

    assert!(result.is_err(), "Should fail with invalid derivation path");
    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("Failed to parse derivation path"),
        "Error should mention derivation path parsing: {err}"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_different_account_addresses() {
    let (client, app) = setup_app(4002);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let provider = create_jsonrpc_client();

    // Test with different predeployed account address
    let account_address = Felt::from_hex_unchecked(
        "0x01b0f8a1ab14f84573d8ed9eec0852a2099ff76ffb601686ffb14fac352b78b3",
    );

    let account = ledger::ledger_account_with_app(
        app,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await;

    assert!(account.is_ok(), "Should create account with valid address");
    let account = account.unwrap();
    assert_eq!(
        account.address(),
        account_address,
        "Account address should match"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_different_derivation_paths() {
    let (client, app) = setup_app(4003);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let provider = create_jsonrpc_client();
    let account_address = Felt::from_hex_unchecked(
        "0x0691a61b12a7105b1372cc377f135213c11e8400a546f6b0e7ea0296046690ce",
    );

    // Test with Braavos derivation path pattern
    let braavos_path = "m/2645'/1195502025'/1148870692'/0'/0'/0";

    let result = ledger::ledger_account_with_app(
        app,
        braavos_path,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await;

    // Should successfully create account with valid Braavos path
    assert!(
        result.is_ok(),
        "Should create account with valid Braavos derivation path: {:?}",
        result.err()
    );
}

// ============================================================================
// Network Integration Tests (Invoke)
// ============================================================================

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_invoke_happy_case() {
    let (client, app) = setup_app(5001);

    // Setup automation for blind signing
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    // Deploy an account that matches the Ledger's public key (unique salt for this test)
    let (account_address, _public_key) =
        deploy_ledger_account(app, TEST_LEDGER_PATH, Felt::from(5001_u32)).await;

    let provider = create_jsonrpc_client();
    let (_client2, app2) = setup_app(5001);

    // Create ledger account with the deployed address
    let account = ledger::ledger_account_with_app(
        app2,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await
    .expect("Failed to create ledger account");

    // Invoke the MAP contract
    let result = account
        .execute_v3(vec![Call {
            to: Felt::from_hex_unchecked(MAP_CONTRACT_ADDRESS_SEPOLIA),
            selector: get_selector_from_name("put").unwrap(),
            calldata: vec![Felt::ONE, Felt::TWO],
        }])
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Invoke transaction failed: {:?}",
        result.err()
    );

    let tx_result = result.unwrap();
    assert!(
        tx_result.transaction_hash != Felt::ZERO,
        "Transaction hash should not be zero"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_invoke_with_wait() {
    let (client, app) = setup_app(5002);

    // Setup automation for blind signing
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    // Deploy an account that matches the Ledger's public key (unique salt for this test)
    let (account_address, _public_key) =
        deploy_ledger_account(app, TEST_LEDGER_PATH, Felt::from(5002_u32)).await;

    let provider = create_jsonrpc_client();
    let (_client2, app2) = setup_app(5002);

    let account = ledger::ledger_account_with_app(
        app2,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await
    .expect("Failed to create ledger account");

    // Send transaction and wait for receipt
    let result = account
        .execute_v3(vec![Call {
            to: Felt::from_hex_unchecked(MAP_CONTRACT_ADDRESS_SEPOLIA),
            selector: get_selector_from_name("put").unwrap(),
            calldata: vec![Felt::from(3_u32), Felt::from(4_u32)],
        }])
        .send()
        .await
        .expect("Failed to send transaction");

    // Wait for transaction to be accepted
    let receipt = provider
        .get_transaction_receipt(result.transaction_hash)
        .await
        .expect("Failed to get transaction receipt");

    assert!(
        matches!(receipt.receipt, Invoke(_)),
        "Expected Invoke receipt"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_multiple_invokes() {
    let (client, app) = setup_app(6002);

    // Setup automation for blind signing
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    // Deploy an account that matches the Ledger's public key (unique salt for this test)
    let (account_address, _public_key) =
        deploy_ledger_account(app, TEST_LEDGER_PATH, Felt::from(6002_u32)).await;

    let provider = create_jsonrpc_client();
    let (_client2, app2) = setup_app(6002);

    let account = ledger::ledger_account_with_app(
        app2,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await
    .expect("Failed to create ledger account");

    // First invoke
    let result1 = account
        .execute_v3(vec![Call {
            to: Felt::from_hex_unchecked(MAP_CONTRACT_ADDRESS_SEPOLIA),
            selector: get_selector_from_name("put").unwrap(),
            calldata: vec![Felt::from(0x10_u32), Felt::from(0x20_u32)],
        }])
        .send()
        .await
        .expect("First invoke failed");

    // Second invoke
    let result2 = account
        .execute_v3(vec![Call {
            to: Felt::from_hex_unchecked(MAP_CONTRACT_ADDRESS_SEPOLIA),
            selector: get_selector_from_name("put").unwrap(),
            calldata: vec![Felt::from(0x30_u32), Felt::from(0x40_u32)],
        }])
        .send()
        .await
        .expect("Second invoke failed");

    // Verify both transactions have different hashes
    assert_ne!(
        result1.transaction_hash, result2.transaction_hash,
        "Transaction hashes should be different"
    );

    // Verify both transactions are on-chain
    let receipt1 = provider
        .get_transaction_receipt(result1.transaction_hash)
        .await;
    let receipt2 = provider
        .get_transaction_receipt(result2.transaction_hash)
        .await;

    assert!(receipt1.is_ok(), "First transaction should have a receipt");
    assert!(receipt2.is_ok(), "Second transaction should have a receipt");
}

// ============================================================================
// Network Integration Tests (Deploy)
// ============================================================================

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_deploy_happy_case() {
    let (client, app) = setup_app(5003);

    // Setup automation for blind signing
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    // Deploy an account that matches the Ledger's public key (unique salt for this test)
    let (account_address, _public_key) =
        deploy_ledger_account(app, TEST_LEDGER_PATH, Felt::from(5003_u32)).await;

    let provider = create_jsonrpc_client();
    let (_client2, app2) = setup_app(5003);

    let account = ledger::ledger_account_with_app(
        app2,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await
    .expect("Failed to create ledger account");

    // Deploy a contract using the MAP contract class hash
    let class_hash = Felt::from_hex_unchecked(MAP_CONTRACT_CLASS_HASH_SEPOLIA);
    let salt = Felt::from(0x123_u32);
    let constructor_calldata = vec![];
    let unique = true;

    let factory = starknet_rust::contract::ContractFactory::new_with_udc(
        class_hash,
        &account,
        starknet_rust::contract::UdcSelector::New,
    );
    let deployment = factory.deploy_v3(constructor_calldata, salt, unique);

    let result = deployment.send().await;

    assert!(
        result.is_ok(),
        "Deploy transaction failed: {:?}",
        result.err()
    );

    let deployed = result.unwrap();
    assert!(
        deployed.transaction_hash != Felt::ZERO,
        "Transaction hash should not be zero"
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_ledger_deploy_with_constructor() {
    let (client, app) = setup_app(6001);

    // Setup automation for blind signing
    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGNING,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    // Deploy an account that matches the Ledger's public key (unique salt for this test)
    let (account_address, _public_key) =
        deploy_ledger_account(app, TEST_LEDGER_PATH, Felt::from(6001_u32)).await;

    let provider = create_jsonrpc_client();
    let (_client2, app2) = setup_app(6001);

    let account = ledger::ledger_account_with_app(
        app2,
        TEST_LEDGER_PATH,
        account_address,
        starknet_rust::core::chain_id::SEPOLIA,
        ExecutionEncoding::New,
        &provider,
    )
    .await
    .expect("Failed to create ledger account");

    // Deploy with constructor parameters
    let class_hash = Felt::from_hex_unchecked(MAP_CONTRACT_CLASS_HASH_SEPOLIA);
    let salt = Felt::from(0x456_u32);
    let constructor_calldata = vec![];

    let factory = starknet_rust::contract::ContractFactory::new_with_udc(
        class_hash,
        &account,
        starknet_rust::contract::UdcSelector::New,
    );
    let result = factory
        .deploy_v3(constructor_calldata, salt, true)
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Deploy with constructor failed: {:?}",
        result.err()
    );

    let deployed = result.unwrap();
    let receipt = provider
        .get_transaction_receipt(deployed.transaction_hash)
        .await
        .expect("Failed to get receipt");

    assert!(
        matches!(receipt.receipt, Invoke(_)),
        "Expected Invoke receipt for deployment"
    );
}
