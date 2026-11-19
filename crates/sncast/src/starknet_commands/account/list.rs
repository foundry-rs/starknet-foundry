use anyhow::{Context, Result};
use clap::Args;
use sncast::accounts::AccountRepository;
use sncast::check_account_file_exists;
use sncast::response::account::list::{AccountDataRepresentation, AccountListResponse};

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
    repository: &AccountRepository,
    display_private_keys: bool,
) -> Result<AccountListResponse> {
    check_account_file_exists(repository)?;

    let accounts_file = repository
        .path()
        .canonicalize_utf8()
        .context("Failed to resolve the accounts file path")?;

    let registry = repository.load()?.registry;
    let mut accounts: Vec<_> = registry
        .networks()
        .iter()
        .flat_map(|(chain_id, chain_accounts)| {
            chain_accounts.iter().map(move |(name, account)| {
                AccountDataRepresentation::new(account, chain_id, name, display_private_keys)
            })
        })
        .collect();
    accounts.sort_unstable_by(|a, b| (&a.chain_id, &a.name).cmp(&(&b.chain_id, &b.name)));

    Ok(AccountListResponse {
        accounts_file,
        accounts,
        display_private_keys,
    })
}
