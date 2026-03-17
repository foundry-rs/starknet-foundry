use anyhow::{Context, Result};
use clap::Args;
use coins_ledger::transports::LedgerAsync;
use starknet_rust::signers::ledger::LedgerStarknetApp;

use sncast::response::ledger::{LedgerResponse, VersionResponse};

#[derive(Args, Debug)]
pub struct AppVersion;

pub async fn app_version<T: LedgerAsync + 'static>(
    _args: &AppVersion,
    ledger: LedgerStarknetApp<T>,
) -> Result<LedgerResponse> {
    let version = ledger
        .get_version()
        .await
        .context("Failed to get app version from Ledger")?;

    Ok(LedgerResponse::Version(VersionResponse {
        version: version.to_string(),
    }))
}
