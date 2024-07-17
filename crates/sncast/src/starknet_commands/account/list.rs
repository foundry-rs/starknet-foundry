use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Args;
use itertools::Itertools;
use serde_json::Value;
use sncast::{
    check_account_file_exists, read_and_parse_json_file, response::print::OutputFormat,
    AccountData, NumbersFormat,
};
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::fmt::{Debug, Display, LowerHex};

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

fn println_numeric_field<T: Debug + Display + LowerHex>(
    title: &'static str,
    format: NumbersFormat,
    item: T,
) {
    match format {
        NumbersFormat::Default | NumbersFormat::Hex => println!("  {title}: {item:#x}"),
        NumbersFormat::Decimal => println!("  {title}: {item}"),
    }
}

fn print_pretty(
    account: &AccountData,
    name: &str,
    network: &str,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
) {
    macro_rules! println_some {
        ($title:expr, $numbers_format:expr, $item:expr) => {
            if let Some(it) = $item {
                println_numeric_field($title, $numbers_format, it);
            }
        };

        ($title:expr, $item:expr) => {
            if let Some(it) = $item {
                println!("  {}: {}", $title, it);
            }
        };
    }

    println!("- {name}:");

    if display_private_keys {
        println_numeric_field("private key", numbers_format, account.private_key);
    };

    println_numeric_field("public key", numbers_format, account.public_key);
    println!("  network: {network}");
    println_some!("address", numbers_format, account.address);
    println_some!("salt", numbers_format, account.salt);
    println_some!("class hash", numbers_format, account.class_hash);
    println_some!("deployed", account.deployed);
    println_some!("legacy", account.legacy);
    println_some!("type", account.account_type);
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
            print_pretty(data, name, network, display_private_keys, numbers_format);
        }
    }

    if !display_private_keys {
        println!("\nTo show private keys too, run with --display-private-keys or -p");
    }
}

type JsonMap = HashMap<String, HashMap<String, HashMap<String, Value>>>;

fn erase_private_keys(networks: &mut JsonMap) {
    for (_, accounts) in networks.iter_mut() {
        for (_, account) in accounts.iter_mut() {
            account.remove("private_key");
        }
    }
}

fn reformat_numeric_fields(networks: &mut JsonMap) -> anyhow::Result<()> {
    macro_rules! replace_some {
        ($account:expr, $field:expr) => {
            if let Some(field) = $account.get_mut($field) {
                let value = FieldElement::from_hex_be(field.as_str().unwrap())
                    .context("Failed to parse account JSON")?;
                *field = Value::String(format!("{value}"));
            }
        };
    }

    for (_, accounts) in networks.iter_mut() {
        for (_, account) in accounts.iter_mut() {
            replace_some!(account, "private_key");
            replace_some!(account, "public_key");
            replace_some!(account, "address");
            replace_some!(account, "salt");
            replace_some!(account, "class_hash");
        }
    }

    Ok(())
}

fn print_as_json(
    networks: &mut JsonMap,
    display_private_keys: bool,
    numbers_format: NumbersFormat,
) -> anyhow::Result<()> {
    if networks.values().all(|net| net.values().len() == 0) {
        println!("{{}}");
        return Ok(());
    }

    if !display_private_keys {
        erase_private_keys(networks);
    }

    if numbers_format == NumbersFormat::Decimal {
        reformat_numeric_fields(networks)?;
    }

    let json = serde_json::to_string_pretty(networks)?;
    print!("{json}");

    Ok(())
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
            let mut networks: JsonMap = read_and_parse_json_file(accounts_file)?;

            print_as_json(&mut networks, display_private_keys, numbers_format)?;
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
