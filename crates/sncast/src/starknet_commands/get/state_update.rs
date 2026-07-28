use anyhow::Result;
use clap::Args;
use sncast::get_block_id;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::get::state_update::StateUpdateResponse;
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get the state update for the given block")]
pub struct StateUpdate {
    /// Block identifier on which the state update should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(default_value = "pre_confirmed")]
    pub id: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn state_update(
    state_update: StateUpdate,
    config: CastConfig,
    ui: &UI,
) -> Result<ExitCode> {
    let provider = state_update.rpc.get_provider(&config, ui).await?;

    let result = get_state_update(&provider, &state_update.id)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get state-update", result, ui, None))
}

async fn get_state_update(
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &str,
) -> Result<StateUpdateResponse, StarknetCommandError> {
    let block_id = get_block_id(block_id)?;
    let state_update = provider
        .get_state_update(block_id)
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;
    Ok(StateUpdateResponse(state_update))
}
