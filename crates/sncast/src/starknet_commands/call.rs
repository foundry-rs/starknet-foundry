use anyhow::Result;
use clap::Args;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::CallResponse;
use starknet::core::types::{BlockId, FunctionCall};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

use crate::Arguments;

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct CallArguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1.., env = "SNCAST_CALL_CALLDATA")]
    pub calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true, env = "SNCAST_CALL_ARGUMENTS")]
    pub arguments: Option<String>,
}

impl From<CallArguments> for Arguments {
    fn from(value: CallArguments) -> Self {
        let CallArguments {
            calldata,
            arguments,
        } = value;
        Self {
            calldata,
            arguments,
        }
    }
}

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct Call {
    /// Address of the called contract (hex)
    #[arg(short = 'd', long, env = "SNCAST_CALL_CONTRACT_ADDRESS")]
    pub contract_address: Felt,

    /// Name of the contract function to be called
    #[arg(short, long, env = "SNCAST_CALL_FUNCTION")]
    pub function: String,

    #[command(flatten)]
    pub arguments: CallArguments,

    /// Block identifier on which call should be performed.
    /// Possible values: pending, latest, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "pending", env = "SNCAST_CALL_BLOCK_ID")]
    pub block_id: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn call(
    contract_address: Felt,
    entry_point_selector: Felt,
    calldata: Vec<Felt>,
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &BlockId,
) -> Result<CallResponse, StarknetCommandError> {
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
