use std::collections::HashMap;

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Args;
use indoc::printdoc;
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

    let repr = networks
        .iter()
        .sorted_by_key(|(name, _)| *name)
        .map(|(name, accounts)| {
            format!(
                "Network \"{}\":\n{}",
                name,
                accounts
                    .iter()
                    .sorted_by_key(|(name, _)| *name)
                    .map(|(name, account)| format!(
                        "- {}:\n{}",
                        name,
                        account.to_string_pretty(display_private_keys)
                    ))
                    .format("\n")
            )
        })
        .format("\n");

    printdoc!(
        "
        Available accounts (at {accounts_file_path}):
        {repr}
        "
    );

    if !display_private_keys {
        println!("\nTo show private keys too, run with --display-private-keys or -p");
    }

    Ok(())
}
