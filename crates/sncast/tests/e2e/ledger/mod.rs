#![cfg(not(target_arch = "wasm32"))]

use std::env;
use std::{borrow::Cow, sync::Arc};

use crate::helpers::constants::URL;
use crate::helpers::fixtures::mint_token;
use serde_json::json;
use sncast::AccountType;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
};
use sncast::helpers::ledger::{create_ledger_app, parse_derivation_path};
use sncast::response::ui::UI;
use speculos_client::{
    AutomationAction, AutomationCondition, AutomationRule, Button, DeviceModel, SpeculosClient,
};
use starknet_rust::accounts::{AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory};
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LedgerSigner;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;
use url::Url;

mod account;
mod docs_snippets;
mod network;
mod simple;

pub(crate) const OZ_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/0'/0";
pub(crate) const READY_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/1'/0";
pub(crate) const BRAAVOS_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/2'/0";
pub(crate) const TEST_LEDGER_PATH: &str = OZ_LEDGER_PATH;
const APP_PATH: &str = "tests/data/ledger-app/nanox#strk#0.25.13.elf";

pub(crate) const LEDGER_PUBLIC_KEY: &str =
    "0x51f3e99d539868d8f45ca705ad6f75e68229a6037a919b15216b4e92a4d6d8";

pub(crate) const LEDGER_ACCOUNT_NAME: &str = "my_ledger";

pub(crate) fn setup_speculos(port: u16) -> (Arc<SpeculosClient>, String) {
    let client = Arc::new(SpeculosClient::new(DeviceModel::Nanox, port, APP_PATH).unwrap());
    let url = format!("http://127.0.0.1:{port}");
    (client, url)
}

fn create_jsonrpc_client() -> JsonRpcClient<HttpTransport> {
    JsonRpcClient::new(HttpTransport::new(Url::parse(URL).unwrap()))
}

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

pub(crate) async fn deploy_ledger_account(speculos_url: &str, path: &str, salt: Felt) -> Felt {
    deploy_ledger_account_of_type(speculos_url, path, salt, AccountType::OpenZeppelin).await
}

async fn deploy_if_needed<F>(
    factory: F,
    salt: Felt,
    provider: &JsonRpcClient<HttpTransport>,
) -> Felt
where
    F: AccountFactory + Sync,
    F::SignError: Send,
{
    let deployment = factory.deploy_v3(salt);
    let address = deployment.address();
    let is_deployed = provider
        .get_class_hash_at(BlockId::Tag(BlockTag::Latest), address)
        .await
        .is_ok();
    mint_token(&format!("{address:#066x}"), u128::MAX).await;
    if !is_deployed {
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

pub(crate) async fn deploy_ledger_account_of_type(
    speculos_url: &str,
    path: &str,
    salt: Felt,
    account_type: AccountType,
) -> Felt {
    let provider = create_jsonrpc_client();
    let ui = UI::default();
    let parsed_path = parse_derivation_path(path, &ui).unwrap();

    // SAFETY: Even with different url, the account will be deployed correctly in devnet.
    unsafe {
        env::set_var("LEDGER_EMULATOR_URL", speculos_url);
    };

    let app = create_ledger_app().await.unwrap();
    let ledger_signer = LedgerSigner::new_with_app(parsed_path, app).unwrap();
    let chain_id = starknet_rust::core::chain_id::SEPOLIA;

    match account_type {
        AccountType::OpenZeppelin => {
            let factory = OpenZeppelinAccountFactory::new(
                OZ_CLASS_HASH,
                chain_id,
                ledger_signer,
                provider.clone(),
            )
            .await
            .unwrap();
            deploy_if_needed(factory, salt, &provider).await
        }
        AccountType::Ready | AccountType::Argent => {
            let factory = ArgentAccountFactory::new(
                READY_CLASS_HASH,
                chain_id,
                None,
                ledger_signer,
                provider.clone(),
            )
            .await
            .unwrap();
            deploy_if_needed(factory, salt, &provider).await
        }
        AccountType::Braavos => {
            let factory = BraavosAccountFactory::new(
                BRAAVOS_CLASS_HASH,
                BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                chain_id,
                ledger_signer,
                provider.clone(),
            )
            .await
            .unwrap();
            deploy_if_needed(factory, salt, &provider).await
        }
    }
}

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
