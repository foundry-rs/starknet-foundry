use anyhow::{Context, Result};
use cast::handle_rpc_error;
use clap::Args;
use starknet::core::types::{BlockId, FieldElement, FunctionCall};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

use cast::parse_number;

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct Call {
    /// Address of the called contract (hex)
    #[clap(short = 'a', long = "contract-address")]
    pub contract_address: String,

    /// Name of the contract function to be called
    #[clap(short = 'f', long = "function-name")]
    pub function_name: String,

    /// Arguments of the called function (list of hex)
    #[clap(short = 'c', long = "calldata", value_delimiter = ' ')]
    pub calldata: Vec<String>,

    /// Block identifier on which call should be performed.
    /// Possible values: pending, latest, block hash (0x prefixed string)
    /// and block number (u64)
    #[clap(short = 'b', long = "block-id", default_value = "pending")]
    pub block_id: String,
}

#[allow(clippy::ptr_arg)]
pub async fn call(
    contract_address: &str,
    func_name: &str,
    calldata: &Vec<String>,
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &BlockId,
) -> Result<Vec<FieldElement>> {
    let function_call = FunctionCall {
        contract_address: parse_number(contract_address)?,
        entry_point_selector: get_selector_from_name(func_name)
            .context("Failed to convert entry point selector to FieldElement")?,
        calldata: calldata
            .iter()
            .map(|x| parse_number(x).context("Failed to convert calldata to FieldElement"))
            .collect::<Result<Vec<_>>>()?,
    };
    let res = provider.call(function_call, block_id).await;

    match res {
        Ok(res) => Ok(res),
        Err(error) => handle_rpc_error(error),
    }
}
