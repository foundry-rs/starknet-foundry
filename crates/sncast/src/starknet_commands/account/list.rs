use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use sncast::response::account::list::{AccountDataRepresentation, AccountListResponse};
use sncast::{AccountData, NestedMap, check_account_file_exists, read_and_parse_json_file};

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

pub fn list(
    accounts_file: &Utf8PathBuf,
    display_private_keys: bool,
) -> Result<AccountListResponse> {
    check_account_file_exists(accounts_file)?;

    let accounts_file = accounts_file
        .canonicalize_utf8()
        .context("Failed to resolve the accounts file path")?;

    let mut accounts: Vec<_> = read_and_parse_json_file::<NestedMap<AccountData>>(&accounts_file)?
        .into_iter()
        .flat_map(|(network, network_accounts)| {
            network_accounts.into_iter().map(move |(name, account)| {
                AccountDataRepresentation::new(account, network.clone(), name, display_private_keys)
            })
        })
        .collect();
    accounts.sort_unstable_by(|a, b| (&a.network, &a.name).cmp(&(&b.network, &b.name)));

    Ok(AccountListResponse {
        accounts_file,
        accounts,
        display_private_keys,
    })
}
