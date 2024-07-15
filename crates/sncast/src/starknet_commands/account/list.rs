use std::collections::HashMap;

use anyhow::anyhow;
use camino::Utf8PathBuf;
use clap::Args;
use indoc::printdoc;
use itertools::Itertools;
use sncast::{check_account_file_exists, read_and_parse_json_file, AccountData};

#[derive(Args, Debug)]
#[command(
    name = "list",
    about = "List available accounts",
    before_help = "Warning! This command exposes cryptographic information, e.g. accounts' private keys"
)]
pub struct List {}

pub fn print_account_list(accounts_file: &Utf8PathBuf) -> anyhow::Result<()> {
    check_account_file_exists(accounts_file)?;

    let accounts_file_path = accounts_file.canonicalize()?;
    let accounts_file_path = accounts_file_path.to_str().ok_or(anyhow!(
        "Failed to resolve an absolute path to the accounts file"
    ))?;

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
                    .map(|(name, account)| format!("- {name}:\n{account}"))
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

    Ok(())
}
