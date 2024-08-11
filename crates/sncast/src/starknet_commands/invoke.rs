use anyhow::{anyhow, Context, Result};
use clap::{Args, ValueEnum};
use sncast::helpers::data_transformer::transform_input_calldata;
use sncast::helpers::error::token_not_supported_for_invoke;
use sncast::helpers::fee::{FeeArgs, FeeSettings, FeeToken, PayableTransaction};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::InvokeResponse;
use sncast::{
    apply_optional, handle_rpc_error, handle_wait_for_tx, impl_payable_transaction, WaitForTx,
};
use starknet::accounts::AccountError::Provider as ProviderErr;
use starknet::accounts::{Account, ConnectedAccount, ExecutionV1, ExecutionV3, SingleOwnerAccount};
use starknet::core::types::{BlockId, BlockTag, Call, Felt, InvokeTransactionResult};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::LocalWallet;

#[derive(Args, Clone)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[clap(short = 'a', long)]
    pub contract_address: Felt,

    /// Name of the function to invoke
    #[clap(short, long)]
    pub function: String,

    /// Calldata for the invoked function - Cairo-like expression
    #[clap(short, long)]
    pub calldata: Option<String>,

    #[clap(flatten)]
    pub fee_args: FeeArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<Felt>,

    /// Version of invoke (can be inferred from fee token)
    #[clap(short, long)]
    pub version: Option<InvokeVersion>,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum InvokeVersion {
    V1,
    V3,
}

impl_payable_transaction!(Invoke, token_not_supported_for_invoke,
    InvokeVersion::V1 => FeeToken::Eth,
    InvokeVersion::V3 => FeeToken::Strk
);

pub async fn invoke(
    invoke: Invoke,
    function_selector: Felt,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse, StarknetCommandError> {
    let fee_args = invoke
        .fee_args
        .clone()
        .fee_token(invoke.token_from_version());

    let transformed_calldata = match invoke.calldata {
        Some(calldata) => {
            let contract_class_hash = account
                .provider()
                .get_class_hash_at(BlockId::Tag(BlockTag::Latest), invoke.contract_address)
                .await
                .map_err(handle_rpc_error)
                .context(format!(
                    "Couldn't retrieve class hash of the contract with address: {:#x}",
                    invoke.contract_address
                ))?;
            transform_input_calldata(
                &calldata,
                &function_selector,
                contract_class_hash,
                account.provider(),
            )
            .await
            .context(format!(
                r#"Failed to serialize input calldata "{calldata}""#
            ))?
        }
        None => vec![],
    };

    let call = Call {
        to: invoke.contract_address,
        selector: function_selector,
        calldata: transformed_calldata,
    };

    execute_calls(account, vec![call], fee_args, invoke.nonce, wait_config).await
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

    let result = match fee_settings {
        FeeSettings::Eth { max_fee } => {
            let execution_calls = account.execute_v1(calls);

            let execution = apply_optional(execution_calls, max_fee, ExecutionV1::max_fee);
            let execution = apply_optional(execution, nonce, ExecutionV1::nonce);
            execution.send().await
        }
        FeeSettings::Strk {
            max_gas,
            max_gas_unit_price,
        } => {
            let execution_calls = account.execute_v3(calls);

            let execution = apply_optional(execution_calls, max_gas, ExecutionV3::gas);
            let execution = apply_optional(execution, max_gas_unit_price, ExecutionV3::gas_price);
            let execution = apply_optional(execution, nonce, ExecutionV3::nonce);
            execution.send().await
        }
    };

    match result {
        Ok(InvokeTransactionResult { transaction_hash }) => handle_wait_for_tx(
            account.provider(),
            transaction_hash,
            InvokeResponse { transaction_hash },
            wait_config,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(ProviderErr(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        _ => Err(anyhow!("Unknown RPC error").into()),
    }
}
