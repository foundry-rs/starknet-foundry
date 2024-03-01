use anyhow::Result;
use clap::Args;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{CallResponse, Felt};
use starknet::core::types::{BlockId, FieldElement, FunctionCall};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct Call {
    /// Address of the called contract (hex)
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the contract function to be called
    #[clap(short, long)]
    pub function: String,

    /// Arguments of the called function (list of hex)
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Vec<FieldElement>,

    /// Block identifier on which call should be performed.
    /// Possible values: pending, latest, block hash (0x prefixed string)
    /// and block number (u64)
    #[clap(short, long, default_value = "pending")]
    pub block_id: String,
}

#[allow(clippy::ptr_arg)]
pub async fn call(
    contract_address: FieldElement,
    entry_point_selector: FieldElement,
    calldata: Vec<FieldElement>,
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &BlockId,
) -> Result<CallResponse, StarknetCommandError> {
    let function_call = FunctionCall {
        contract_address,
        entry_point_selector,
        calldata,
    };
    let res = provider
        .call(function_call, block_id)
        .await
        .map(|v| v.into_iter().map(Felt).collect());

    match res {
        Ok(response) => Ok(CallResponse { response }),
        Err(error) => Err(StarknetCommandError::ProviderError(error.into())),
    }
}
