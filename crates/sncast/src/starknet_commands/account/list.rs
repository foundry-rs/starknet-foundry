use anyhow::Error;
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::IntoDecStr;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sncast::AccountType;
use sncast::NumbersFormat;
use sncast::{AccountData, NestedMap, check_account_file_exists, read_and_parse_json_file};
use std::collections::HashMap;
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
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
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
    fn new(
        account: &AccountData,
        display_private_key: bool,
        numbers_format: NumbersFormat,
    ) -> Self {
        match numbers_format {
            NumbersFormat::Default | NumbersFormat::Hex => Self {
                private_key: display_private_key.then(|| account.private_key.into_hex_string()),
                public_key: account.public_key.into_hex_string(),
                network: None,
                address: account.address.map(IntoHexStr::into_hex_string),
                salt: account.salt.map(IntoHexStr::into_hex_string),
                deployed: account.deployed,
                class_hash: account.class_hash.map(IntoHexStr::into_hex_string),
                legacy: account.legacy,
                account_type: account.account_type,
            },
            NumbersFormat::Decimal => Self {
                private_key: display_private_key.then(|| account.private_key.into_dec_string()),
                public_key: account.public_key.into_dec_string(),
                network: None,
                address: account.address.map(IntoDecStr::into_dec_string),
                salt: account.salt.map(IntoDecStr::into_dec_string),
                deployed: account.deployed,
                class_hash: account.class_hash.map(IntoDecStr::into_dec_string),
                legacy: account.legacy,
                account_type: account.account_type,
            },
        }
    }

    fn set_network(&mut self, network: &str) {
        self.network = Some(network.to_owned());
    }
}

fn read_and_flatten(
    accounts_file: &Utf8PathBuf,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
) -> anyhow::Result<HashMap<String, AccountDataRepresentationMessage>> {
    let networks: NestedMap<AccountData> = read_and_parse_json_file(accounts_file)?;
    let mut result = HashMap::new();

    for (network, accounts) in networks.iter().sorted_by_key(|(name, _)| *name) {
        for (name, data) in accounts.iter().sorted_by_key(|(name, _)| *name) {
            let mut data_repr =
                AccountDataRepresentationMessage::new(data, display_private_keys, numbers_format);

            data_repr.set_network(network);
            result.insert(name.to_owned(), data_repr);
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

        if let Some(ref private_key) = self.private_key {
            let _ = writeln!(result, "  private key: {private_key}");
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
            let _ = writeln!(result, "  type: {account_type}");
        }

        result.trim_end().to_string()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

#[derive(Serialize)]
pub struct AccountsListMessage {
    accounts_file: Utf8PathBuf,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
}

impl AccountsListMessage {
    pub fn new(
        accounts_file: Utf8PathBuf,
        display_private_keys: bool,
        numbers_format: NumbersFormat,
    ) -> Result<Self, Error> {
        check_account_file_exists(&accounts_file)?;
        Ok(Self {
            accounts_file,
            display_private_keys,
            numbers_format,
        })
    }
}

impl Message for AccountsListMessage {
    fn text(&self) -> String {
        let accounts_file_path = self
            .accounts_file
            .canonicalize()
            .expect("Failed to resolve the accounts file path");

        let accounts_file_path = accounts_file_path
            .to_str()
            .expect("Failed to resolve an absolute path to the accounts file");

        let accounts = read_and_flatten(
            &self.accounts_file,
            self.display_private_keys,
            self.numbers_format,
        )
        .unwrap_or_default();

        if accounts.is_empty() {
            format!("No accounts available at {accounts_file_path}")
        } else {
            let mut result = format!("Available accounts (at {accounts_file_path}):");
            for (name, data) in accounts.iter().sorted_by_key(|(name, _)| *name) {
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
        let accounts = read_and_flatten(
            &self.accounts_file,
            self.display_private_keys,
            self.numbers_format,
        )
        .unwrap_or_default();

        let mut accounts_map: HashMap<String, AccountDataRepresentationMessage> = HashMap::new();
        for (name, data) in &accounts {
            accounts_map.insert(name.clone(), data.clone());
        }

        json!(&accounts_map)
    }
}
