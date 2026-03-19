use anyhow::Result;
use clap::Args;
use sncast::get_block_id;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::nonce::NonceResponse;
use sncast::response::ui::UI;
use std::process::ExitCode;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

#[derive(Debug, Args)]
#[command(about = "Get the nonce of a contract")]
pub struct Nonce {
    /// Address of the contract
    pub contract_address: Felt,

    /// Block identifier on which nonce should be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pre_confirmed")]
    pub block_id: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn nonce(nonce: Nonce, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = nonce.rpc.get_provider(&config, ui).await?;

    let result = get_nonce(&provider, nonce.contract_address, &nonce.block_id).await;

    Ok(process_command_result("get nonce", result, ui, None))
}

pub async fn get_nonce(
    provider: &JsonRpcClient<HttpTransport>,
    contract_address: Felt,
    block_id: &str,
) -> Result<NonceResponse> {
    let block_id = get_block_id(block_id)?;
    let nonce = provider
        .get_nonce(block_id, contract_address)
        .await
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;
    Ok(NonceResponse { nonce })
}
