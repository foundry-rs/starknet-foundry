use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::transaction::TransactionResponse;
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

#[derive(Debug, Args)]
#[command(about = "Get the details of a transaction")]
pub struct Transaction {
    /// Hash of the transaction
    pub transaction_hash: Felt,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn transaction(tx: Transaction, config: CastConfig, ui: &UI) -> Result<()> {
    let provider = tx.rpc.get_provider(&config, ui).await?;

    let result = get_transaction(&provider, tx.transaction_hash).await;

    let chain_id = provider.chain_id().await?;
    let block_explorer_link = block_explorer_link_if_allowed(&result, chain_id, &config).await;

    process_command_result("get tx", result, ui, block_explorer_link);
    Ok(())
}

async fn get_transaction(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
) -> Result<TransactionResponse> {
    let response = provider
        .get_transaction_by_hash(transaction_hash)
        .await
        .map(TransactionResponse)
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(response)
}
