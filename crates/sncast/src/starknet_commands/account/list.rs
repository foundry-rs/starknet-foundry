use anyhow::Error;
use clap::Args;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use itertools::Itertools;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sncast::accounts::schema::v2;
use sncast::accounts::{AccountRecord, AccountRepository};
use sncast::signers::SignerSpec;
use sncast::{AccountType, check_account_file_exists};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Args, Debug)]
#[command(
    name = "list",
    about = "List available accounts",
    before_help = "Warning! This command may expose vulnerable cryptographic information, e.g. accounts' private keys"
)]
pub struct List {
    /// Display private keys
    #[arg(short = 'p', long = "display-private-keys")]
    pub display_private_keys: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct AccountDataRepresentationMessage {
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer: Option<v2::Signer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,
    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,
}

impl AccountDataRepresentationMessage {
    fn new(account: &AccountRecord, display_private_key: bool) -> Self {
        Self {
            signer: match &account.signer {
                SignerSpec::PrivateKey(_) if !display_private_key => None,
                SignerSpec::PrivateKey(spec) => Some(v2::Signer::PrivateKey {
                    private_key: spec.private_key(),
                }),
                SignerSpec::Keystore(spec) => Some(v2::Signer::Keystore {
                    path: spec.path().clone(),
                    password_env: spec.password_env().map(ToOwned::to_owned),
                }),
                SignerSpec::Ledger(spec) => Some(v2::Signer::Ledger {
                    derivation_path: spec.derivation_path().derivation_string(),
                }),
            },
            public_key: account.public_key.into_hex_string(),
            network: None,
            address: account.address.map(IntoHexStr::into_hex_string),
            salt: account.salt.map(IntoHexStr::into_hex_string),
            deployed: account.deployed,
            class_hash: account.class_hash.map(IntoHexStr::into_hex_string),
            legacy: account.legacy,
            account_type: account.account_type,
        }
    }

    fn set_network(&mut self, network: &str) {
        self.network = Some(network.to_owned());
    }
}

fn read_and_flatten(
    repository: &AccountRepository,
    display_private_keys: bool,
) -> anyhow::Result<BTreeMap<String, AccountDataRepresentationMessage>> {
    let registry = repository.load()?.registry;
    let mut result = BTreeMap::new();

    for (network, accounts) in registry.networks() {
        for (name, data) in accounts.iter().sorted_by_key(|(name, _)| *name) {
            let mut data_repr = AccountDataRepresentationMessage::new(data, display_private_keys);

            data_repr.set_network(network.as_str());
            result.insert(name.to_string(), data_repr);
        }
    }

    Ok(result)
}

impl Message for AccountDataRepresentationMessage {
    fn text(&self) -> String {
        let mut result = String::new();

        if let Some(ref network) = self.network {
            let _ = writeln!(result, "  network: {network}");
        }

        let _ = writeln!(result, "  public key: {}", self.public_key);

        match &self.signer {
            Some(v2::Signer::PrivateKey { private_key }) => {
                let _ = writeln!(result, "  private key: {}", private_key.into_hex_string());
            }
            Some(v2::Signer::Ledger { derivation_path }) => {
                let _ = writeln!(result, "  ledger path: {derivation_path}");
            }
            Some(v2::Signer::Keystore { path, password_env }) => {
                let _ = writeln!(result, "  keystore: {path}");
                if let Some(password_env) = password_env {
                    let _ = writeln!(result, "  password env: {password_env}");
                }
            }
            None => {}
        }
        if let Some(ref address) = self.address {
            let _ = writeln!(result, "  address: {address}");
        }
        if let Some(ref salt) = self.salt {
            let _ = writeln!(result, "  salt: {salt}");
        }
        if let Some(ref class_hash) = self.class_hash {
            let _ = writeln!(result, "  class hash: {class_hash}");
        }
        if let Some(ref deployed) = self.deployed {
            let _ = writeln!(result, "  deployed: {deployed}");
        }
        if let Some(ref legacy) = self.legacy {
            let _ = writeln!(result, "  legacy: {legacy}");
        }
        if let Some(ref account_type) = self.account_type {
            let _ = writeln!(result, "  type: {account_type:?}");
        }

        result.trim_end().to_string()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

pub struct AccountsListMessage {
    accounts_file_path: String,
    display_private_keys: bool,
    accounts: BTreeMap<String, AccountDataRepresentationMessage>,
}

impl AccountsListMessage {
    pub fn new(repository: AccountRepository, display_private_keys: bool) -> Result<Self, Error> {
        check_account_file_exists(repository.path())?;

        let accounts_file_path = repository
            .path()
            .canonicalize()
            .expect("Failed to resolve the accounts file path");

        let accounts_file_path = accounts_file_path
            .to_str()
            .expect("Failed to resolve an absolute path to the accounts file");
        let accounts = read_and_flatten(&repository, display_private_keys)?;

        Ok(Self {
            accounts_file_path: accounts_file_path.to_string(),
            display_private_keys,
            accounts,
        })
    }
}

impl Message for AccountsListMessage {
    fn text(&self) -> String {
        if self.accounts.is_empty() {
            format!("No accounts available at {}", self.accounts_file_path)
        } else {
            let mut result = format!("Available accounts (at {}):", self.accounts_file_path);
            for (name, data) in &self.accounts {
                let _ = writeln!(result, "\n- {}:\n{}", name, data.text());
            }
            if !self.display_private_keys {
                let _ = writeln!(
                    result,
                    "\nTo show private keys too, run with --display-private-keys or -p"
                );
            }
            result
        }
    }

    fn json(&self) -> Value {
        json!(&self.accounts)
    }
}
