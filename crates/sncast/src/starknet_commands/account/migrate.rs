use anyhow::{Result, ensure};
use clap::Args;

use sncast::accounts::AccountRepository;
use sncast::response::account::migrate::AccountMigrateResponse;

#[derive(Args, Debug)]
#[command(about = "Migrate an accounts file to the latest schema version")]
pub struct Migrate;

pub fn migrate(repository: &AccountRepository) -> Result<AccountMigrateResponse> {
    ensure!(
        repository.file_exists(),
        "Failed to migrate accounts file at {}; the file doesn't exist",
        repository.path()
    );

    repository
        .update_to_latest_schema()
        .map(AccountMigrateResponse::from)
        .map_err(anyhow::Error::from)
}
