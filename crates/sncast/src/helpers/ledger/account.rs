use super::{SncastLedgerTransport, create_ledger_app};
use crate::response::ui::UI;
use anyhow::{Context, Result, bail};
use starknet_rust::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::types::{BlockId, BlockTag},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{DerivationPath, LedgerSigner},
};
use starknet_types_core::felt::Felt;

const LEDGER_SIGNER_ERROR: &str = "Failed to create Ledger signer. Ensure the derivation path is correct and the Ledger app is ready.";

pub async fn create_ledger_signer(
    ledger_path: &DerivationPath,
    ui: &UI,
    print_message: bool,
) -> Result<LedgerSigner<SncastLedgerTransport>> {
    let ledger_app = create_ledger_app().await?;

    if print_message {
        ui.print_notification(
            "Ledger device will display a confirmation screen. Please approve it to continue...\n",
        );
    }

    LedgerSigner::new_with_app(ledger_path.clone(), ledger_app).context(LEDGER_SIGNER_ERROR)
}

pub fn verify_ledger_public_key(ledger_public_key: Felt, stored_public_key: Felt) -> Result<()> {
    if ledger_public_key != stored_public_key {
        bail!(
            "Public key mismatch!\n\
            Ledger public key: {ledger_public_key:#x}\n\
            Stored public key: {stored_public_key:#x}\n\
            \n\
            This account was created with a different Ledger derivation path or public key.\n\
            Make sure you're using the same derivation path that was used during account creation."
        );
    }
    Ok(())
}

pub async fn ledger_account<'a>(
    ledger_path: &DerivationPath,
    address: Felt,
    chain_id: Felt,
    encoding: ExecutionEncoding,
    provider: &'a JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LedgerSigner<SncastLedgerTransport>>>
{
    let signer = create_ledger_signer(ledger_path, ui, true).await?;

    let mut account = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
    account.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));

    Ok(account)
}

pub async fn get_ledger_public_key(ledger_path: &DerivationPath, ui: &UI) -> Result<Felt> {
    let ledger_app = create_ledger_app().await?;

    ui.print_notification("Please confirm the public key on your Ledger device...\n");

    let public_key = ledger_app
        .get_public_key(ledger_path.clone(), true)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(public_key.scalar())
}
