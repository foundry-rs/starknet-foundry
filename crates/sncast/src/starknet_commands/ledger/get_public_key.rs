use anyhow::{Context, Result};
use clap::Args;
use coins_ledger::transports::LedgerAsync;
use conversions::string::IntoHexStr;
use sncast::helpers::ledger::LedgerKeyLocator;
use sncast::response::ui::UI;
use starknet_rust::signers::ledger::LedgerStarknetApp;

use sncast::response::ledger::{LedgerResponse, PublicKeyResponse};

#[derive(Args, Debug)]
pub struct GetPublicKey {
    #[command(flatten)]
    pub key_locator: LedgerKeyLocator,

    /// Do not display the public key on Ledger's screen for confirmation
    #[arg(long)]
    pub no_display: bool,
}

pub async fn get_public_key<T: LedgerAsync + 'static>(
    args: &GetPublicKey,
    ledger: LedgerStarknetApp<T>,
    ui: &UI,
) -> Result<LedgerResponse> {
    let path = args.key_locator.resolve(ui);

    if !args.no_display {
        ui.print_notification("Please confirm the public key on your Ledger device...\n");
    }

    let public_key = ledger
        .get_public_key(path, !args.no_display)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(LedgerResponse::PublicKey(PublicKeyResponse {
        public_key: public_key.scalar().into_hex_string(),
    }))
}
