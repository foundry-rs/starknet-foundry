use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::transaction::TransactionResponse;
use sncast::response::ui::UI;
use starknet_rust::core::types::TransactionResponseFlag;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get the details of a transaction")]
pub struct Transaction {
    #[allow(clippy::struct_field_names)]
    /// Hash of the transaction
    pub transaction_hash: Felt,

    /// Include proof facts in transaction response
    #[arg(long)]
    pub with_proof_facts: bool,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn transaction(tx: Transaction, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = tx.rpc.get_provider(&config, ui).await?;

    let result = get_transaction(&provider, tx.transaction_hash, tx.with_proof_facts)
        .await
        .map_err(handle_starknet_command_error);

    let chain_id = provider.chain_id().await?;
    let block_explorer_link = block_explorer_link_if_allowed(&result, chain_id, &config).await;

    Ok(process_command_result(
        "get tx",
        result,
        ui,
        block_explorer_link,
    ))
}

async fn get_transaction(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
    with_proof_facts: bool,
) -> Result<TransactionResponse, StarknetCommandError> {
    let response_flags = if with_proof_facts {
        Some(&[TransactionResponseFlag::IncludeProofFacts][..])
    } else {
        None
    };
    let response = provider
        .get_transaction_by_hash(transaction_hash, response_flags)
        .await
        .map(TransactionResponse)
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(response)
}
