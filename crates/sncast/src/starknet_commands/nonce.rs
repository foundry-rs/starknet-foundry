use anyhow::Result;
use clap::Args;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{SNCastProviderError, StarknetCommandError};
use sncast::response::nonce::NonceResponse;
use sncast::get_block_id;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

#[derive(Args)]
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

pub async fn nonce(
    provider: &JsonRpcClient<HttpTransport>,
    contract_address: Felt,
    block_id: &str,
) -> Result<NonceResponse> {
    let block_id = get_block_id(block_id)?;
    let nonce = provider
        .get_nonce(block_id, contract_address)
        .await
        .map_err(|err| StarknetCommandError::ProviderError(SNCastProviderError::from(err)))?;
    Ok(NonceResponse { nonce })
}
