use std::collections::HashMap;

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Args;
use itertools::Itertools;
use sncast::{check_account_file_exists, read_and_parse_json_file, AccountData};

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

fn print_pretty(account: &AccountData, name: &str, network: &str, display_private_keys: bool) {
    macro_rules! println_some {
        ( $format_spec:expr, $item:expr) => {
            if let Some(it) = $item {
                println!($format_spec, it);
            }
        };
    }

    println!("- {name}:");

    if display_private_keys {
        println!("  private key: {:#x}", account.private_key);
    };

    println!("  public key: {:#x}", account.public_key);
    println!("  network: {network}");
    println_some!("  address: {:#x}", account.address);
    println_some!("  salt: {:#x}", account.salt);
    println_some!("  class hash: {:#x}", account.class_hash);
    println_some!("  deployed: {}", account.deployed);
    println_some!("  legacy: {}", account.legacy);
    println_some!("  type: {}", account.account_type);
    println!();
}

pub fn print_account_list(
    accounts_file: &Utf8PathBuf,
    display_private_keys: bool,
) -> anyhow::Result<()> {
    check_account_file_exists(accounts_file)?;

    let accounts_file_path = accounts_file.canonicalize()?;
    let accounts_file_path = accounts_file_path
        .to_str()
        .context("Failed to resolve an absolute path to the accounts file")?;

    let networks: HashMap<String, HashMap<String, AccountData>> =
        read_and_parse_json_file(accounts_file)?;

    if networks.values().all(|net| net.values().len() == 0) {
        println!("No accounts available at {accounts_file_path}");
        return Ok(());
    }

    println!("Available accounts (at {accounts_file_path}):");

    for (network, accounts) in networks.iter().sorted_by_key(|(name, _)| *name) {
        for (name, data) in accounts.iter().sorted_by_key(|(name, _)| *name) {
            print_pretty(data, name, network, display_private_keys);
        }
    }

    if !display_private_keys {
        println!("\nTo show private keys too, run with --display-private-keys or -p");
    }

    Ok(())
}
