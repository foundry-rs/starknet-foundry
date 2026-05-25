use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::tx_receipt::TxReceiptResponse;
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get the receipt of a transaction")]
pub struct TxReceipt {
    #[allow(clippy::struct_field_names)]
    /// Hash of the transaction
    pub transaction_hash: Felt,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn tx_receipt(tx: TxReceipt, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = tx.rpc.get_provider(&config, ui).await?;

    let result = get_tx_receipt(&provider, tx.transaction_hash)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get tx-receipt", result, ui, None))
}

async fn get_tx_receipt(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
) -> Result<TxReceiptResponse, StarknetCommandError> {
    let response = provider
        .get_transaction_receipt(transaction_hash)
        .await
        .map(TxReceiptResponse)
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(response)
}
