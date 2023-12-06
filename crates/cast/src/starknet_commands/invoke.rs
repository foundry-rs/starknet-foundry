use anyhow::{anyhow, Result};
use clap::Args;

use cast::helpers::response_structs::InvokeResponse;
use cast::{handle_rpc_error, handle_wait_for_tx, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

#[derive(Args)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the function to invoke
    #[clap(short, long)]
    pub function: String,

    /// Calldata for the invoked function
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub calldata: Vec<FieldElement>,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<FieldElement>,
}

pub async fn invoke(
    contract_address: FieldElement,
    entry_point_name: &str,
    calldata: Vec<FieldElement>,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let call = Call {
        to: contract_address,
        selector: get_selector_from_name(entry_point_name)?,
        calldata,
    };

    execute_calls(account, vec![call], max_fee, nonce, wait_config).await
}

pub async fn execute_calls(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    calls: Vec<Call>,
    max_fee: Option<FieldElement>,
    nonce: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let execution_calls = account.execute(calls);

    // todo: refactor setting max_fee and nonce
    let execution_with_fee = if let Some(max_fee) = max_fee {
        execution_calls.max_fee(max_fee)
    } else {
        execution_calls
    };

    let execution = if let Some(nonce) = nonce {
        execution_with_fee.nonce(nonce)
    } else {
        execution_with_fee
    };

    match execution.send().await {
        Ok(result) => {
            handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                InvokeResponse {
                    transaction_hash: result.transaction_hash,
                },
                wait_config,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
