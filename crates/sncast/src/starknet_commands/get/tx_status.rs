use anyhow::Result;
use clap::Args;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::tx_status::{ExecutionStatus, FinalityStatus, TransactionStatusResponse};
use sncast::response::ui::UI;
use starknet_rust::core::types::{TransactionExecutionStatus, TransactionStatus};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;

#[derive(Debug, Args)]
#[command(about = "Get the status of a transaction")]
pub struct TxStatus {
    /// Hash of the transaction
    pub transaction_hash: Felt,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn tx_status(tx_status: TxStatus, config: CastConfig, ui: &UI) -> anyhow::Result<()> {
    let provider = tx_status.rpc.get_provider(&config, ui).await?;

    let result = get_tx_status(&provider, tx_status.transaction_hash)
        .await
        .map_err(handle_starknet_command_error);

    process_command_result("get tx-status", result, ui, None);
    Ok(())
}

pub async fn get_tx_status(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: Felt,
) -> Result<TransactionStatusResponse, StarknetCommandError> {
    let response = provider
        .get_transaction_status(transaction_hash)
        .await
        .map(|status| build_transaction_status_response(&status))
        .map_err(|err| StarknetCommandError::ProviderError(err.into()))?;

    Ok(response)
}

fn build_transaction_status_response(status: &TransactionStatus) -> TransactionStatusResponse {
    match status {
        TransactionStatus::Received => TransactionStatusResponse {
            finality_status: FinalityStatus::Received,
            execution_status: None,
        },
        TransactionStatus::Candidate => TransactionStatusResponse {
            finality_status: FinalityStatus::Candidate,
            execution_status: None,
        },
        TransactionStatus::PreConfirmed(tx_exec_result) => TransactionStatusResponse {
            finality_status: FinalityStatus::PreConfirmed,
            execution_status: Some(build_execution_status(tx_exec_result.status())),
        },
        TransactionStatus::AcceptedOnL2(tx_exec_result) => TransactionStatusResponse {
            finality_status: FinalityStatus::AcceptedOnL2,
            execution_status: Some(build_execution_status(tx_exec_result.status())),
        },
        TransactionStatus::AcceptedOnL1(tx_exec_result) => TransactionStatusResponse {
            finality_status: FinalityStatus::AcceptedOnL1,
            execution_status: Some(build_execution_status(tx_exec_result.status())),
        },
    }
}

fn build_execution_status(status: TransactionExecutionStatus) -> ExecutionStatus {
    match status {
        TransactionExecutionStatus::Succeeded => ExecutionStatus::Succeeded,
        TransactionExecutionStatus::Reverted => ExecutionStatus::Reverted,
    }
}
