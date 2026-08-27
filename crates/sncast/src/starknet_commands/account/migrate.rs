use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Args;

use sncast::accounts::AccountRepository;
use sncast::response::account::migrate::AccountMigrateResponse;

#[derive(Args, Debug)]
#[command(about = "Migrate an accounts file to the latest schema version")]
pub struct Migrate;

pub fn migrate(accounts_file: &Utf8PathBuf) -> Result<AccountMigrateResponse> {
    AccountRepository::new(accounts_file.to_owned())?
        .update_to_latest_schema()
        .map(AccountMigrateResponse::from)
        .map_err(anyhow::Error::from)
}
