use anyhow::Error;
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sncast::AccountType;
use sncast::{
    AccountData, NestedMap, SignerType, check_account_file_exists, read_and_parse_json_file,
};
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

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccountDataRepresentationMessage {
    pub name: String,
    pub public_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub signer_type: Option<SignerType>,
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
    fn new(account: AccountData, name: String, network: String, display_private_key: bool) -> Self {
        let AccountData {
            public_key,
            address,
            salt,
            deployed,
            class_hash,
            legacy,
            account_type,
            signer_type,
        } = account;

        let signer_type = match signer_type {
            SignerType::Local { .. } if !display_private_key => None,
            signer_type => Some(signer_type),
        };

        Self {
            name,
            signer_type,
            public_key: public_key.into_hex_string(),
            network: Some(network),
            address: address.map(IntoHexStr::into_hex_string),
            salt: salt.map(IntoHexStr::into_hex_string),
            deployed,
            class_hash: class_hash.map(IntoHexStr::into_hex_string),
            legacy,
            account_type,
        }
    }
}

impl Message for AccountDataRepresentationMessage {
    fn text(&self) -> String {
        let mut result = String::new();

        if let Some(ref network) = self.network {
            let _ = writeln!(result, "  network: {network}");
        }

        let _ = writeln!(result, "  public key: {}", self.public_key);

        match &self.signer_type {
            Some(SignerType::Local { private_key }) => {
                let _ = writeln!(result, "  private key: {}", private_key.into_hex_string());
            }
            Some(SignerType::Ledger { ledger_path }) => {
                let _ = writeln!(result, "  ledger path: {}", ledger_path.derivation_string());
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
    accounts_file_path: Utf8PathBuf,
    accounts: BTreeMap<String, Vec<AccountDataRepresentationMessage>>,
    display_private_keys: bool,
}

impl AccountsListMessage {
    pub fn try_new(accounts_file: &Utf8PathBuf, display_private_keys: bool) -> Result<Self, Error> {
        check_account_file_exists(accounts_file)?;

        let accounts_file_path = accounts_file
            .canonicalize_utf8()
            .expect("Failed to resolve the accounts file path");

        let accounts = read_accounts(&accounts_file_path, display_private_keys)?;

        Ok(Self {
            accounts_file_path,
            accounts,
            display_private_keys,
        })
    }
}

impl Message for AccountsListMessage {
    fn text(&self) -> String {
        if self.accounts.is_empty() {
            return format!("No accounts available at {}", self.accounts_file_path);
        }

        let mut result = format!("Available accounts (at {}):", self.accounts_file_path);

        for network_accounts in self.accounts.values() {
            for account in network_accounts {
                let _ = writeln!(result, "\n- {}:\n{}", account.name, account.text());
            }
        }

        if !self.display_private_keys {
            let _ = writeln!(
                result,
                "\nTo show private keys too, run with --display-private-keys or -p"
            );
        }

        result
    }

    fn json(&self) -> Value {
        json!(&self.accounts)
    }
}

fn read_accounts(
    accounts_file: &Utf8PathBuf,
    display_private_keys: bool,
) -> anyhow::Result<BTreeMap<String, Vec<AccountDataRepresentationMessage>>> {
    let accounts_for_networks = read_and_parse_json_file::<NestedMap<AccountData>>(accounts_file)?
        .into_iter()
        .map(|(network_name, network_accounts)| {
            let accounts = network_accounts
                .into_iter()
                .map(|(account_name, account_data)| {
                    AccountDataRepresentationMessage::new(
                        account_data,
                        account_name,
                        network_name.clone(),
                        display_private_keys,
                    )
                })
                .sorted_by(|repr1, repr2| repr1.name.cmp(&repr2.name))
                .collect();

            (network_name, accounts)
        })
        .sorted_by(|(network_name1, _), (network_name2, _)| network_name1.cmp(network_name2))
        .collect();

    Ok(accounts_for_networks)
}
