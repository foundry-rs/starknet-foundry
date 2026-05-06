#![cfg(not(target_arch = "wasm32"))]

use std::{borrow::Cow, sync::Arc};

use crate::helpers::constants::URL;
use crate::helpers::fixtures::mint_token;
use clap::Command;
use clap::builder::TypedValueParser;
use serde_json::json;
use sncast::AccountType;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
};
use sncast::helpers::ledger::{DerivationPathParser, SncastLedgerTransport};
use sncast::response::ui::UI;
use speculos::{AutomationAction, AutomationCondition, AutomationRule, Button, SpeculosClient};
use starknet_rust::accounts::{AccountFactory, ArgentAccountFactory, OpenZeppelinAccountFactory};
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LedgerSigner;
use starknet_rust::signers::ledger::LedgerStarknetApp;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;
use url::Url;

mod account;
mod basic;
mod network;
pub(crate) mod speculos;

pub(crate) const OZ_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/0'/0";
pub(crate) const READY_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/1'/0";
pub(crate) const BRAAVOS_LEDGER_PATH: &str = "m//starknet'/sncast'/0'/2'/0";
pub(crate) const TEST_LEDGER_PATH: &str = OZ_LEDGER_PATH;

pub(crate) const TEST_LEDGER_PATH_STORED: &str = "m/2645'/1195502025'/355113700'/0'/0'/0";

const APP_PATH: &str = "tests/data/ledger-app/nanox.elf";

pub(crate) const LEDGER_PUBLIC_KEY: &str =
    "0x51f3e99d539868d8f45ca705ad6f75e68229a6037a919b15216b4e92a4d6d8";

pub(crate) const LEDGER_ACCOUNT_NAME: &str = "my_ledger";

pub(crate) fn setup_speculos(port: u16) -> (Arc<SpeculosClient>, String) {
    let client = Arc::new(SpeculosClient::new(port, APP_PATH).unwrap());
    let url = format!("http://127.0.0.1:{port}");
    (client, url)
}

/// Sets automation rules and, when `ENABLE_BLIND_SIGN` is among them, presses RIGHT so the
/// blind-sign flow advances immediately.
pub(crate) async fn set_automation(
    client: &SpeculosClient,
    rules: &[speculos::AutomationRule<'static>],
) {
    client.automation(rules).await.unwrap();
    let needs_blind_sign = rules.iter().any(|r| r == &automation::ENABLE_BLIND_SIGN);
    if needs_blind_sign {
        client.click_button(Button::Right).await.unwrap();
    }
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
                "ledger_path": TEST_LEDGER_PATH_STORED,
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
        deployment.send().await.expect("Failed to deploy account");
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
    let parsed = DerivationPathParser
        .parse_ref(&Command::new("test"), None, std::ffi::OsStr::new(path))
        .unwrap();
    parsed.print_warnings(&ui);
    let parsed_path = parsed.path;

    let transport = SncastLedgerTransport::new(speculos_url.to_string()).unwrap();
    let app = LedgerStarknetApp::from_transport(transport);
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

    // Screen flow: "Public Key (1/2)" -> Right -> "Public Key (2/2)" -> Right -> "Approve" -> Both
    // Trigger fires on "Public Key (1/2)" and navigates to Approve, then confirms.
    pub(crate) const APPROVE_PUBLIC_KEY: AutomationRule<'static> = AutomationRule {
        text: Some(Cow::Borrowed("Public Key (1/2)")),
        regexp: None,
        x: None,
        y: None,
        conditions: &[],
        actions: &[
            // Right (to "Public Key (2/2)")
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right (to "Approve")
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (confirm)
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

    // Screen flow: navigate to "App settings" (via press_button RIGHT from home) ->
    // Both (enter settings, shows "Blind signing" toggle OFF) -> Both (toggle ON).
    // The trigger is "App settings" so the caller must press RIGHT after registering this
    // rule to navigate away from the home screen and land on "App settings".
    pub(crate) const ENABLE_BLIND_SIGN: AutomationRule<'static> = AutomationRule {
        text: Some(Cow::Borrowed("App settings")),
        regexp: None,
        x: None,
        y: None,
        conditions: &[AutomationCondition {
            varname: Cow::Borrowed("blind_enabled"),
            value: false,
        }],
        actions: &[
            // Both (enter settings, shows "Blind signing" toggle OFF)
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
            // Both (toggle blind signing ON)
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
            // Mark as done
            AutomationAction::Setbool {
                varname: Cow::Borrowed("blind_enabled"),
                value: true,
            },
        ],
    };

    // Screen flow: "Blind signing ahead." -> Both -> "Review hash" -> Right -> "Hash (1/2)" ->
    // Right -> "Hash (2/2)" -> Right -> "Sign Hash ?" -> Both -> "Message signed"
    /// Must be used with [`ENABLE_BLIND_SIGN`].
    pub(crate) const APPROVE_BLIND_SIGN_HASH: AutomationRule<'static> = AutomationRule {
        text: None,
        regexp: Some(Cow::Borrowed("^Blind signing ahead")),
        x: None,
        y: None,
        conditions: &[AutomationCondition {
            varname: Cow::Borrowed("blind_enabled"),
            value: true,
        }],
        actions: &[
            // Both (accept "Blind signing ahead" warning)
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
            // Right (to "Hash (1/2)")
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right (to "Hash (2/2)")
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Right (to "Sign Hash ?")
            AutomationAction::Button {
                button: Button::Right,
                pressed: true,
            },
            AutomationAction::Button {
                button: Button::Right,
                pressed: false,
            },
            // Both (confirm)
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
