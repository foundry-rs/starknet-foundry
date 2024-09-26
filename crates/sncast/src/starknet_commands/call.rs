use anyhow::{Context, Result};
use clap::Args;
use sncast::helpers::data_transformer::transformer::transform;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::CallResponse;
use sncast::{get_class_hash_by_address, get_contract_class};
use starknet::core::types::{BlockId, Felt, FunctionCall};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct Call {
    /// Address of the called contract (hex)
    #[clap(short = 'a', long)]
    pub contract_address: Felt,

    /// Name of the contract function to be called
    #[clap(short, long)]
    pub function: String,

    /// Arguments of the called function, either entirely serialized or entirely written as Cairo-like expression strings
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Vec<String>,

    /// Block identifier on which call should be performed.
    /// Possible values: pending, latest, block hash (0x prefixed string)
    /// and block number (u64)
    #[clap(short, long, default_value = "pending")]
    pub block_id: String,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[allow(clippy::ptr_arg)]
pub async fn call(
    contract_address: Felt,
    entry_point_selector: Felt,
    calldata: Vec<String>,
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &BlockId,
) -> Result<CallResponse, StarknetCommandError> {
    let class_hash = get_class_hash_by_address(provider, contract_address)
        .await?
        .with_context(|| {
            format!("Couldn't retrieve class hash of a contract with address {contract_address:#x}")
        })?;

    let contract_class = get_contract_class(class_hash, provider).await?;

    let calldata = transform(&calldata, contract_class, &entry_point_selector)?;

    let function_call = FunctionCall {
        contract_address,
        entry_point_selector,
        calldata,
    };

    let res = provider.call(function_call, block_id).await;

    match res {
        Ok(response) => Ok(CallResponse { response }),
        Err(error) => Err(StarknetCommandError::ProviderError(error.into())),
    }
}
