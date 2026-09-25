use anyhow::Result;
use clap::Args;
use sncast::{
    helpers::{command::process_command_result, configuration::CastConfig, rpc::RpcArgs},
    response::{
        errors::{StarknetCommandError, handle_starknet_command_error},
        get::syncing::SyncingResponse,
        ui::UI,
    },
};
use starknet_rust::providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport};
use std::process::ExitCode;

#[derive(Debug, Args)]
pub struct Syncing {
    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn syncing(syncing: Syncing, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = syncing.rpc.get_provider(&config, ui).await?;

    let result = get_syncing_status(&provider)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get syncing", result, ui, None))
}

#[expect(clippy::result_large_err)]
async fn get_syncing_status(
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<SyncingResponse, StarknetCommandError> {
    let sync_status = provider
        .syncing()
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    let status = match sync_status {
        starknet_rust::core::types::SyncStatusType::Syncing(sync_status) => Some(sync_status),
        starknet_rust::core::types::SyncStatusType::NotSyncing => None,
    };

    Ok(SyncingResponse {
        syncing: status.is_some(),
        status,
    })
}
