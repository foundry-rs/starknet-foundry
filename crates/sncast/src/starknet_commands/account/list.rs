use std::collections::HashMap;

use camino::Utf8PathBuf;
use clap::Args;
use itertools::Itertools;
use sncast::{check_account_file_exists, read_and_parse_json_file, AccountData};

#[derive(Args, Debug)]
#[command(about = "List available accounts")]
pub struct List {}

pub fn list(accounts_file: &Utf8PathBuf) -> anyhow::Result<()> {
    check_account_file_exists(accounts_file)?;

    let networks: HashMap<String, HashMap<String, AccountData>> =
        read_and_parse_json_file(accounts_file)?;

    let no_networks = networks.len() == 0;
    let no_accounts = networks.values().all(|net| net.values().len() == 0);

    if no_networks || no_accounts {
        println!("No accounts available");
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

    print!("Available accounts:\n{repr}");
    Ok(())
}
