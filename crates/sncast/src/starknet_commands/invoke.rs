use anyhow::{Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use foundry_ui::UI;
use sncast::helpers::arguments::Arguments;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::invoke::InvokeResponse;
use sncast::{WaitForTx, apply_optional_fields, handle_wait_for_tx};
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

#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub async fn invoke(
    contract_address: Felt,
    calldata: Vec<Felt>,
    nonce: Option<Felt>,
    fee_args: FeeArgs,
    function_selector: Felt,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse, StarknetCommandError> {
    let call = Call {
        to: contract_address,
        selector: function_selector,
        calldata,
    };

    execute_calls(account, vec![call], fee_args, nonce, wait_config, ui).await
}

pub async fn execute_calls(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    calls: Vec<Call>,
    fee_args: FeeArgs,
    nonce: Option<Felt>,
    wait_config: WaitForTx,
    ui: &UI,
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
            ui,
        )
        .await
        .map_err(StarknetCommandError::from),
        Err(Provider(error)) => Err(StarknetCommandError::ProviderError(error.into())),
        Err(error) => Err(anyhow!(format!("Unexpected error occurred: {error}")).into()),
    }
}
