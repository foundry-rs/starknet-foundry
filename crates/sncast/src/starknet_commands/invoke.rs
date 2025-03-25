use crate::Arguments;
use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::InvokeResponse;
use sncast::{WaitForTx, apply_optional, handle_wait_for_tx};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, ConnectedAccount, ExecutionV3, SingleOwnerAccount};
use starknet::core::types::{Call, InvokeTransactionResult};
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::LocalWallet;
use starknet_types_core::felt::Felt;

#[derive(Args, Clone, Debug)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[arg(short = 'd', long)]
    pub contract_address: Felt,

    /// Name of the function to invoke
    #[arg(short, long)]
    pub function: String,

    #[command(flatten)]
    pub arguments: Arguments,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
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
    let fee_settings = fee_args
        .try_into_fee_settings(account.provider(), account.block_id())
        .await?;

    let FeeSettings {
        max_gas,
        max_gas_unit_price,
    } = fee_settings;
    let execution_calls = account.execute_v3(calls);

    let execution = apply_optional(
        execution_calls,
        max_gas.map(std::num::NonZero::get),
        ExecutionV3::gas,
    );
    let execution = apply_optional(
        execution,
        max_gas_unit_price.map(std::num::NonZero::get),
        ExecutionV3::gas_price,
    );
    let execution = apply_optional(execution, nonce, ExecutionV3::nonce);
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
