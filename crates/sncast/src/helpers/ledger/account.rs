use super::parse_derivation_path;
use super::{SncastLedgerTransport, create_ledger_app};
use crate::response::ui::UI;
use anyhow::{Context, Result, bail};
use starknet_rust::signers::Signer;
use starknet_rust::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::types::{BlockId, BlockTag},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LedgerSigner,
};
use starknet_types_core::felt::Felt;

const LEDGER_SIGNER_ERROR: &str = "Failed to create Ledger signer. Ensure the derivation path is correct and the Ledger app is ready.";

pub async fn create_ledger_signer(
    ledger_path: &str,
    print_message: bool,
    ui: &UI,
) -> Result<LedgerSigner<SncastLedgerTransport>> {
    let ledger_app = create_ledger_app().await?;
    let path = parse_derivation_path(ledger_path, ui)
        .with_context(|| format!("Failed to parse derivation path '{ledger_path}'"))?;

    if print_message {
        ui.print_notification("Connected to Ledger device\n");
    }

    LedgerSigner::new_with_app(path, ledger_app).context(LEDGER_SIGNER_ERROR)
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
    ledger_path: &str,
    address: Felt,
    chain_id: Felt,
    encoding: ExecutionEncoding,
    stored_public_key: Felt,
    provider: &'a JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LedgerSigner<SncastLedgerTransport>>>
{
    let signer = create_ledger_signer(ledger_path, true, ui).await?;

    let ledger_public_key = signer.get_public_key().await?.scalar();
    verify_ledger_public_key(ledger_public_key, stored_public_key)?;

    let mut account = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
    account.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));

    Ok(account)
}

pub async fn get_ledger_public_key(
    ledger_path: &str,
    display_on_device: bool,
    ui: &UI,
) -> Result<Felt> {
    let ledger_app = create_ledger_app().await?;
    let path = parse_derivation_path(ledger_path, ui)?;

    if display_on_device {
        ui.print_notification("Please confirm the public key on your Ledger device...\n");
    }

    let public_key = ledger_app
        .get_public_key(path, display_on_device)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(public_key.scalar())
}
