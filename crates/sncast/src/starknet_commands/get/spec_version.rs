use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::spec_version::SpecVersionResponse;
use sncast::response::ui::UI;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use std::process::ExitCode;

#[derive(Debug, Args)]
#[command(about = "Get the version of the Starknet JSON-RPC specification used by the node")]
pub struct SpecVersion {
    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn spec_version(
    spec_version: SpecVersion,
    config: CastConfig,
    ui: &UI,
) -> Result<ExitCode> {
    let provider = spec_version.rpc.get_provider(&config, ui).await?;

    let result = get_spec_version(&provider)
        .await
        .map_err(handle_starknet_command_error);

    Ok(process_command_result("get spec-version", result, ui, None))
}

pub async fn get_spec_version(
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<SpecVersionResponse, StarknetCommandError> {
    let spec_version = provider
        .spec_version()
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;
    Ok(SpecVersionResponse { spec_version })
}
