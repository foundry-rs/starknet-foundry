use anyhow::{Context, Result};
use clap::Args;
use sncast::response::structs::TransactionStatusResponse;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Get the status of a transaction")]
pub struct TxStatus {
    /// Hash of the transaction
    pub transaction_hash: FieldElement,
}

pub async fn tx_status(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> Result<TransactionStatusResponse> {
    provider
        .get_transaction_status(transaction_hash)
        .await
        .context("Failed to get transaction status")
        .map(Into::into)
}
