use anyhow::{Context, Result};
use clap::Args;
use coins_ledger::transports::LedgerAsync;
use conversions::string::IntoHexStr;
use foundry_ui::components::warning::WarningMessage;
use sncast::helpers::ledger::LedgerKeyLocator;
use sncast::response::ui::UI;
use starknet_rust::signers::ledger::LedgerStarknetApp;
use starknet_types_core::felt::Felt;

use sncast::response::ledger::{LedgerResponse, SignatureResponse};

#[derive(Args, Debug)]
pub struct SignHash {
    #[command(flatten)]
    pub key_locator: LedgerKeyLocator,

    /// The raw hash to be signed
    pub hash: Felt,
}

pub async fn sign_hash<T: LedgerAsync + 'static>(
    args: &SignHash,
    ledger: LedgerStarknetApp<T>,
    ui: &UI,
) -> Result<LedgerResponse> {
    let path = args.key_locator.resolve(ui);

    ui.print_warning(WarningMessage::new(
        "Blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes \
        from trusted sources. For better security, sign full transactions instead \
        of raw hashes whenever possible.",
    ));
    ui.print_blank_line();

    ui.print_notification("Please confirm the signing operation on your Ledger\n");

    let signature = ledger
        .sign_hash(path, &args.hash)
        .await
        .context("Failed to sign hash with Ledger")?;

    Ok(LedgerResponse::Signature(SignatureResponse {
        r: signature.r.into_hex_string(),
        s: signature.s.into_hex_string(),
    }))
}
