use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Args;
use conversions::string::IntoDecStr;
use conversions::string::IntoHexStr;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use sncast::AccountType;
use sncast::{
    check_account_file_exists, read_and_parse_json_file, response::print::OutputFormat,
    AccountData, NumbersFormat,
};
use std::collections::HashMap;
use std::fmt::Display;

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
pub struct AccountDataRepresentation {
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

impl AccountDataRepresentation {
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

type NestedMap<T> = HashMap<String, HashMap<String, T>>;

fn represent(
    data: &NestedMap<AccountData>,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
) -> NestedMap<AccountDataRepresentation> {
    data.iter()
        .map(|(network, accounts)| {
            let accounts = accounts
                .iter()
                .map(|(name, account)| {
                    (
                        name.to_owned(),
                        AccountDataRepresentation::new(
                            account,
                            display_private_keys,
                            numbers_format,
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();

            (network.to_owned(), accounts)
        })
        .collect::<HashMap<_, _>>()
}

fn print_as_json(networks: &NestedMap<AccountDataRepresentation>) -> anyhow::Result<()> {
    if networks.values().all(|net| net.values().len() == 0) {
        println!("{{}}");
        return Ok(());
    }

    let json = serde_json::to_string_pretty(networks)?;
    print!("{json}");

    Ok(())
}

fn print_if_some<T: Display>(title: &str, item: &Option<T>) {
    if let Some(ref item) = item {
        println!("  {title}: {item}");
    }
}

fn print_pretty(data: &AccountDataRepresentation, name: &str) {
    println!("- {name}:");
    print_if_some("network", &data.network);
    print_if_some("private key", &data.private_key);
    println!("  public key: {}", data.public_key);
    print_if_some("address", &data.address);
    print_if_some("salt", &data.salt);
    print_if_some("class hash", &data.class_hash);
    print_if_some("deployed", &data.deployed);
    print_if_some("legacy", &data.legacy);
    print_if_some("type", &data.account_type);
    println!();
}

fn print_as_human(
    networks: &HashMap<String, HashMap<String, AccountData>>,
    accounts_file_path: &str,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
) {
    if networks.values().all(|net| net.values().len() == 0) {
        println!("No accounts available at {accounts_file_path}");
        return;
    }

    println!("Available accounts (at {accounts_file_path}):");

    for (network, accounts) in networks.iter().sorted_by_key(|(name, _)| *name) {
        for (name, data) in accounts.iter().sorted_by_key(|(name, _)| *name) {
            let mut data_repr =
                AccountDataRepresentation::new(data, display_private_keys, numbers_format);

            data_repr.set_network(network);
            print_pretty(&data_repr, name);
        }
    }

    if !display_private_keys {
        println!("\nTo show private keys too, run with --display-private-keys or -p");
    }
}

pub fn print_account_list(
    accounts_file: &Utf8PathBuf,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
    output_format: &OutputFormat,
) -> anyhow::Result<()> {
    check_account_file_exists(accounts_file)?;

    let accounts_file_path = accounts_file.canonicalize()?;
    let accounts_file_path = accounts_file_path
        .to_str()
        .context("Failed to resolve an absolute path to the accounts file")?;

    match output_format {
        OutputFormat::Json => {
            let networks: NestedMap<AccountData> = read_and_parse_json_file(accounts_file)?;
            let networks = represent(&networks, display_private_keys, numbers_format);

            print_as_json(&networks)?;
        }
        OutputFormat::Human => {
            let networks: HashMap<String, HashMap<String, AccountData>> =
                read_and_parse_json_file(accounts_file)?;

            print_as_human(
                &networks,
                accounts_file_path,
                display_private_keys,
                numbers_format,
            );
        }
    }

    Ok(())
}
