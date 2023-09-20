use anyhow::{Context, Result};
use cast::handle_rpc_error;
use cast::helpers::response_structs::CallResponse;
use clap::Args;
use starknet::core::types::{BlockId, FieldElement, FunctionCall};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Call a contract instance on Starknet", long_about = None)]
pub struct Call {
    /// Address of the called contract (hex)
    #[clap(long, visible_alias = "addr")]
    pub contract_address: FieldElement,

    /// Name of the contract function to be called
    #[clap(short, long)]
    pub function: String,

    /// Arguments of the called function (list of hex)
    #[clap(short, long, value_delimiter = ' ')]
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
    func_name: &str,
    calldata: Vec<FieldElement>,
    provider: &JsonRpcClient<HttpTransport>,
    block_id: &BlockId,
) -> Result<CallResponse> {
    let function_call = FunctionCall {
        contract_address,
        entry_point_selector: get_selector_from_name(func_name)
            .context("Failed to convert entry point selector to FieldElement")?,
        calldata,
    };
    let res = provider.call(function_call, block_id).await;

    match res {
        Ok(res) => {
            let response: String = res.iter().map(|item| format!("{item:#x}, ")).collect();
            Ok(CallResponse {
                response: "[".to_string()
                    + response.trim_end_matches(|c| c == ' ' || c == ',')
                    + "]",
            })
        }
        Err(error) => handle_rpc_error(error),
    }
}
