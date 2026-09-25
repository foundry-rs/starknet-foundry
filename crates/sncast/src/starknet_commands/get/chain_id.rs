use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::get::chain_id::ChainIdResponse;
use sncast::{
    helpers::{configuration::CastConfig, rpc::RpcArgs},
    response::ui::UI,
};
use starknet_rust::core::utils::parse_cairo_short_string;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct ChainId {
    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn chain_id(chain_id: ChainId, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = chain_id.rpc.get_provider(&config, ui).await?;
    let result = get_chain_id(&provider)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get chain-id", result, ui, None))
}

#[expect(clippy::result_large_err)]
async fn get_chain_id(
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<ChainIdResponse, StarknetCommandError> {
    let chain_id = provider
        .chain_id()
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(ChainIdResponse {
        chain_name: parse_cairo_short_string(&chain_id).ok(),
        chain_id,
    })
}
