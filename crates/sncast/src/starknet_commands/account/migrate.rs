use anyhow::Result;
use clap::Args;

use sncast::accounts::AccountRepository;
use sncast::response::account::migrate::AccountMigrateResponse;

#[derive(Args, Debug)]
#[command(about = "Migrate an accounts file to the latest schema version")]
pub struct Migrate;

pub fn migrate(repository: &AccountRepository) -> Result<AccountMigrateResponse> {
    repository
        .update_to_latest_schema()
        .map(AccountMigrateResponse::from)
        .map_err(anyhow::Error::from)
}
