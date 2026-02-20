use super::parse_derivation_path;
use super::{SncastLedgerTransport, create_ledger_app};
use crate::response::ui::UI;
use anyhow::{Context, Result};
use coins_ledger::transports::LedgerAsync;
use starknet_rust::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::types::{BlockId, BlockTag},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{LedgerSigner, ledger::LedgerStarknetApp},
};
use starknet_types_core::felt::Felt;

const LEDGER_SIGNER_ERROR: &str = "Failed to create Ledger signer. Ensure the derivation path is correct and the Ledger app is ready.";

fn create_signer_from_app<T: LedgerAsync + 'static>(
    ledger_app: LedgerStarknetApp<T>,
    ledger_path: &str,
) -> Result<LedgerSigner<T>> {
    let path = parse_derivation_path(ledger_path)
        .with_context(|| format!("Failed to parse derivation path '{ledger_path}'"))?;
    LedgerSigner::new_with_app(path, ledger_app).context(LEDGER_SIGNER_ERROR)
}

pub async fn ledger_account<'a>(
    ledger_path: &str,
    address: Felt,
    chain_id: Felt,
    encoding: ExecutionEncoding,
    provider: &'a JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LedgerSigner<SncastLedgerTransport>>>
{
    let signer = create_ledger_signer(ledger_path, ui).await?;

    let mut account = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
    account.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));

    Ok(account)
}

pub async fn ledger_account_with_app<'a, T>(
    ledger_app: LedgerStarknetApp<T>,
    ledger_path: &str,
    address: Felt,
    chain_id: Felt,
    encoding: ExecutionEncoding,
    provider: &'a JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LedgerSigner<T>>>
where
    T: LedgerAsync + 'static + Send + Sync,
{
    let signer = create_signer_from_app(ledger_app, ledger_path)?;
    ui.print_notification("Connected to Ledger device".to_string());

    let mut account = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
    account.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));

    Ok(account)
}

pub async fn get_ledger_public_key(ledger_path: &str, display_on_device: bool) -> Result<Felt> {
    let ledger_app = create_ledger_app().await?;
    let path = parse_derivation_path(ledger_path)?;

    let public_key = ledger_app
        .get_public_key(path, display_on_device)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(public_key.scalar())
}

pub async fn create_ledger_signer(
    ledger_path: &str,
    ui: &UI,
) -> Result<LedgerSigner<SncastLedgerTransport>> {
    let ledger_app = create_ledger_app().await?;
    let signer = create_signer_from_app(ledger_app, ledger_path)?;
    ui.print_notification("Connected to Ledger device".to_string());
    Ok(signer)
}
