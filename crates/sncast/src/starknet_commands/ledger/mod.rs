use anyhow::Result;
use clap::{Args, Subcommand};
use sncast::helpers::ledger::create_ledger_app;
use sncast::response::ui::UI;

use sncast::response::ledger::LedgerResponse;

use app_version::{AppVersion, app_version};
use get_public_key::{GetPublicKey, get_public_key};
use sign_hash::{SignHash, sign_hash};

pub mod app_version;
pub mod get_public_key;
pub mod sign_hash;

#[derive(Args, Debug)]
#[command(about = "Interact with Ledger hardware wallet")]
pub struct Ledger {
    #[command(subcommand)]
    subcommand: LedgerSubcommand,
}

#[derive(Subcommand, Debug)]
enum LedgerSubcommand {
    /// Get public key from Ledger device
    GetPublicKey(GetPublicKey),
    /// Sign a hash using Ledger device
    SignHash(SignHash),
    /// Get Starknet app version from Ledger device
    AppVersion(AppVersion),
}

pub async fn ledger(ledger_args: &Ledger, ui: &UI) -> Result<LedgerResponse> {
    let ledger = create_ledger_app().await?;

    match &ledger_args.subcommand {
        LedgerSubcommand::GetPublicKey(args) => get_public_key(args, ledger, ui).await,
        LedgerSubcommand::SignHash(args) => sign_hash(args, ledger, ui).await,
        LedgerSubcommand::AppVersion(args) => app_version(args, ledger).await,
    }
}
