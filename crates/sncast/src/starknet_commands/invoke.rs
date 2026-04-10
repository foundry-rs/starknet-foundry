use crate::Arguments;
use crate::starknet_commands::utils::felt_or_id::FeltOrId;
use anyhow::{Context, Result, anyhow};
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::proof::ProofArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::invoke::InvokeResponse;
use sncast::response::ui::UI;
use sncast::{WaitForTx, apply_optional_fields, handle_wait_for_tx};
use starknet_rust::accounts::AccountError::Provider;
use starknet_rust::accounts::{Account, ConnectedAccount, ExecutionV3, SingleOwnerAccount};
use starknet_rust::core::types::{Call, InvokeTransactionResult};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::Signer;
use starknet_types_core::felt::Felt;

#[derive(Args, Clone, Debug)]
pub struct InvokeCommonArgs {
    /// Address of contract to invoke
    #[arg(short = 'd', long)]
    pub contract_address: FeltOrId,

    /// Name of the function to invoke
    #[arg(short, long)]
    pub function: String,

    #[command(flatten)]
    pub arguments: Arguments,
}

#[derive(Args, Clone, Debug)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    #[command(flatten)]
    pub common: InvokeCommonArgs,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub proof_args: ProofArgs,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[arg(short, long)]
    pub nonce: Option<Felt>,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[expect(clippy::too_many_arguments)]
pub async fn invoke<S>(
    contract_address: Felt,
    calldata: Vec<Felt>,
    nonce: Option<Felt>,
    fee_args: FeeArgs,
    proof_args: ProofArgs,
    function_selector: Felt,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, S>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse, StarknetCommandError>
where
    S: Signer + Sync + Send,
{
    let call = Call {
        to: contract_address,
        selector: function_selector,
        calldata,
    };

    execute_calls(
        account,
        vec![call],
        fee_args,
        proof_args,
        nonce,
        wait_config,
        ui,
    )
    .await
}

pub async fn execute_calls<S>(
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, S>,
    calls: Vec<Call>,
    fee_args: FeeArgs,
    proof_args: ProofArgs,
    nonce: Option<Felt>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse, StarknetCommandError>
where
    S: Signer + Sync + Send,
{
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
        tip,
    } = fee_settings.expect("Failed to convert to fee settings");

    let proof = proof_args
        .resolve_proof()
        .context("Failed to resolve proof")?;
    let proof_facts = proof_args
        .resolve_proof_facts()
        .context("Failed to resolve proof facts")?;

    let execution = apply_optional_fields!(
        execution_calls,
        l1_gas => ExecutionV3::l1_gas,
        l1_gas_price => ExecutionV3::l1_gas_price,
        l2_gas => ExecutionV3::l2_gas,
        l2_gas_price => ExecutionV3::l2_gas_price,
        l1_data_gas => ExecutionV3::l1_data_gas,
        l1_data_gas_price => ExecutionV3::l1_data_gas_price,
        tip => ExecutionV3::tip,
        nonce => ExecutionV3::nonce,
        proof => ExecutionV3::proof,
        proof_facts => ExecutionV3::proof_facts
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
