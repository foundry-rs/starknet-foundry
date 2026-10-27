use anyhow::{Result, ensure};
use camino::Utf8Path;
use clap::Args;

use sncast::accounts::AccountRepository;
use sncast::response::account::migrate::AccountMigrateResponse;

#[derive(Args, Debug)]
#[command(about = "Migrate an accounts file to the latest schema version")]
pub struct Migrate;

pub fn migrate(accounts_file: &Utf8Path) -> Result<AccountMigrateResponse> {
    let repository = AccountRepository::try_new(accounts_file)?;

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
