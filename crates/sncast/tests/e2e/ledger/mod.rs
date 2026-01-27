#![cfg(not(target_arch = "wasm32"))]

use std::env;
use std::{borrow::Cow, sync::Arc};

use crate::helpers::constants::URL;
use crate::helpers::fixtures::mint_token;
use serde_json::json;
use sncast::helpers::constants::OZ_CLASS_HASH;
use sncast::ledger::{create_ledger_app, parse_derivation_path};
use speculos_client::{
    AutomationAction, AutomationCondition, AutomationRule, Button, DeviceModel, SpeculosClient,
};
use starknet_rust::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::core::utils::get_contract_address;
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LedgerSigner;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;
use url::Url;

mod account;
mod network;
mod simple;

pub(crate) const TEST_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/0'/0";
const APP_PATH: &str = "tests/data/ledger-app/nanox#strk#0.25.13.elf";

pub(crate) const LEDGER_PUBLIC_KEY: &str =
    "0x51f3e99d539868d8f45ca705ad6f75e68229a6037a919b15216b4e92a4d6d8";

pub(crate) const LEDGER_ACCOUNT_NAME: &str = "my_ledger";

/// Start a Speculos emulator on `port` and return the client and the URL to connect to it.
pub(crate) fn setup_speculos(port: u16) -> (Arc<SpeculosClient>, String) {
    let client = Arc::new(SpeculosClient::new(DeviceModel::Nanox, port, APP_PATH).unwrap());
    let url = format!("http://127.0.0.1:{port}");
    (client, url)
}

fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    JsonRpcClient::new(HttpTransport::new(Url::parse(URL).unwrap()))
}

/// Create a temporary accounts.json file containing a deployed ledger account at `address`.
pub(crate) fn create_temp_accounts_json(address: Felt) -> TempDir {
    let tempdir = TempDir::new().unwrap();
    let accounts_json = json!({
        "alpha-sepolia": {
            LEDGER_ACCOUNT_NAME: {
                "public_key": LEDGER_PUBLIC_KEY,
                "address": format!("{address:#066x}"),
                "deployed": true,
                "type": "open_zeppelin",
                "ledger_path": TEST_LEDGER_PATH,
            }
        }
    });
    let accounts_path = tempdir.path().join("accounts.json");
    std::fs::write(&accounts_path, accounts_json.to_string()).unwrap();
    tempdir
}

/// Create a temporary accounts.json file containing an undeployed ledger account.
/// Returns the account address and the tempdir.
pub(crate) fn create_undeployed_accounts_json(salt: Felt) -> (Felt, TempDir) {
    let public_key = Felt::from_hex(LEDGER_PUBLIC_KEY).unwrap();
    let address = get_contract_address(salt, OZ_CLASS_HASH, &[public_key], Felt::ZERO);

    let tempdir = TempDir::new().unwrap();
    let accounts_json = json!({
        "alpha-sepolia": {
            LEDGER_ACCOUNT_NAME: {
                "public_key": LEDGER_PUBLIC_KEY,
                "address": format!("{address:#x}"),
                "deployed": false,
                "type": "open_zeppelin",
                "class_hash": format!("{OZ_CLASS_HASH:#x}"),
                "salt": format!("{salt:#x}"),
                "ledger_path": TEST_LEDGER_PATH,
            }
        }
    });
    let accounts_path = tempdir.path().join("accounts.json");
    std::fs::write(&accounts_path, accounts_json.to_string()).unwrap();
    (address, tempdir)
}

/// Helper function to deploy an account that matches the Ledger's public key.
/// Uses a unique salt per test to avoid address collisions.
pub(crate) async fn deploy_ledger_account(speculos_url: &str, path: &str, salt: Felt) -> Felt {
    let provider = create_jsonrpc_client();

    let parsed_path = parse_derivation_path(path).unwrap();

    // SAFETY: Even with different url, the account will be deployed correctly in devnet.
    unsafe {
        env::set_var("LEDGER_EMULATOR_URL", speculos_url);
    };

    let app = create_ledger_app().await.unwrap();
    let ledger_signer = LedgerSigner::new_with_app(parsed_path, app).unwrap();

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
        mint_token(&format!("{address:#066x}"), u128::MAX).await;
    } else {
        mint_token(&format!("{address:#066x}"), u128::MAX).await;

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

    address
}

// Automation Rules for Ledger UI Navigation
pub(crate) mod automation {
    use super::*;

    pub(crate) const APPROVE_PUBLIC_KEY: AutomationRule<'static> = AutomationRule {
        text: Some(Cow::Borrowed("Confirm Public Key")),
        regexp: None,
        x: None,
        y: None,
        conditions: &[],
        actions: &[
            // Press right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Press right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Press right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Press both
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
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both
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
            // Both
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
            // Left
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

    /// Must be used with [`ENABLE_BLIND_SIGN`].
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
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both
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
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both
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
