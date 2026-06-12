use anyhow::Result;
use clap::Args;
use sncast::get_block_id;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::block::BlockResponse;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get a block with transaction hashes")]
pub struct Block {
    /// Block identifier on which the block should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(default_value = "latest")]
    pub block_id: String,

    /// Retrieve full transactions instead of only their hashes
    #[arg(long)]
    pub full: bool,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn block(block: Block, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = block.rpc.get_provider(&config, ui).await?;

    let result = get_block(&provider, &block.block_id, block.full)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get block", result, ui, None))
}

async fn get_block(
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &str,
    full: bool,
) -> Result<BlockResponse, StarknetCommandError> {
    let block_id = get_block_id(block_id)?;

    let block = if full {
        provider
            .get_block_with_txs(block_id, None)
            .await
            .map(BlockResponse::WithTxs)
    } else {
        provider
            .get_block_with_tx_hashes(block_id)
            .await
            .map(BlockResponse::WithTxHashes)
    }
    .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(block)
}
