use anyhow::{anyhow, Result};
use clap::Args;

use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{Felt, InvokeResponse};
use sncast::{apply_optional, handle_wait_for_tx, WaitForTx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, ConnectedAccount, ExecutionV1, SingleOwnerAccount};
use starknet::core::types::FieldElement;
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
    function_selector: FieldElement,
    calldata: Vec<FieldElement>,
    max_fee: Option<FieldElement>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    nonce: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, StarknetCommandError> {
    let call = Call {
        to: contract_address,
        selector: function_selector,
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
) -> Result<InvokeResponse, StarknetCommandError> {
    let execution_calls = account.execute_v1(calls);

    let execution = apply_optional(execution_calls, max_fee, ExecutionV1::max_fee);
    let execution = apply_optional(execution, nonce, ExecutionV1::nonce);

    match execution.send().await {
        Ok(result) => handle_wait_for_tx(
            account.provider(),
            result.transaction_hash,
            InvokeResponse {
                transaction_hash: Felt(result.transaction_hash),
            },
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        _ => Err(anyhow!("Unknown RPC error").into()),
    }
}
