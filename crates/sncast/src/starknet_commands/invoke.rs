use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::InvokeResponse;
use sncast::{WaitForTx, apply_optional_fields, handle_wait_for_tx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, ExecutionV3, SingleOwnerAccount};
use starknet::core::types::{Call, InvokeTransactionResult};
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;

use crate::Arguments;

#[derive(Debug, Clone, clap::Args)]
#[group(multiple = false)]
pub struct InvokeArguments {
    /// Arguments of the called function serialized as a series of felts
    #[arg(short, long, value_delimiter = ' ', num_args = 1.., env = "SNCAST_INVOKE_CALLDATA")]
    pub calldata: Option<Vec<String>>,

    // Arguments of the called function as a comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true, env = "SNCAST_INVOKE_ARGUMENTS")]
    pub arguments: Option<String>,
}

impl From<InvokeArguments> for Arguments {
    fn from(value: InvokeArguments) -> Self {
        let InvokeArguments {
            calldata,
            arguments,
        } = value;
        Self {
            calldata,
            arguments,
        }
    }
}

#[derive(Args, Clone, Debug)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[arg(short = 'd', long, env = "SNCAST_INVOKE_CONTRACT_ADDRESS")]
    pub contract_address: Felt,

    /// Name of the function to invoke
    #[arg(short, long, env = "SNCAST_INVOKE_FUNCTION")]
    pub function: String,

    #[command(flatten)]
    pub arguments: InvokeArguments,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long, env = "SNCAST_INVOKE_NONCE")]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn invoke(
    contract_address: Felt,
    calldata: Vec<Felt>,
    nonce: Option<Felt>,
    fee_args: FeeArgs,
    function_selector: Felt,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, StarknetCommandError> {
    let call = Call {
        to: contract_address,
        selector: function_selector,
        calldata,
    };

    execute_calls(account, vec![call], fee_args, nonce, wait_config).await
}

pub async fn execute_calls(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    calls: Vec<Call>,
    fee_args: FeeArgs,
    nonce: Option<Felt>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, StarknetCommandError> {
    let execution_calls = account.execute_v3(calls);

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = execution_calls
            .estimate_fee()
            .await
            .expect("Failed to estimate fee");
        fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
    } = fee_settings.expect("Failed to convert to fee settings");

    let execution = apply_optional_fields!(
        execution_calls,
        l1_gas => ExecutionV3::l1_gas,
        l1_gas_price => ExecutionV3::l1_gas_price,
        l2_gas => ExecutionV3::l2_gas,
        l2_gas_price => ExecutionV3::l2_gas_price,
        l1_data_gas => ExecutionV3::l1_data_gas,
        l1_data_gas_price => ExecutionV3::l1_data_gas_price,
        nonce => ExecutionV3::nonce
    );
    let result = execution.send().await;

    match result {
        Ok(InvokeTransactionResult { transaction_hash }) => handle_wait_for_tx(
            account.provider(),
            transaction_hash,
            InvokeResponse {
                transaction_hash: transaction_hash.into_(),
            },
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
