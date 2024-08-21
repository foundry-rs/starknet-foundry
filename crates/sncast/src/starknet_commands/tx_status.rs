use clap::Args;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::StarknetCommandError;
use sncast::response::structs::{ExecutionStatus, FinalityStatus, TransactionStatusResponse};
use starknet::core::types::{FieldElement, TransactionExecutionStatus, TransactionStatus};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Get the status of a transaction")]
pub struct TxStatus {
    /// Hash of the transaction
    pub transaction_hash: FieldElement,

    #[clap(flatten)]
    pub rpc: RpcArgs,
}

pub async fn tx_status(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> Result<TransactionStatusResponse, StarknetCommandError> {
    provider
        .get_transaction_status(transaction_hash)
        .await
        .map(|status| build_transaction_status_response(&status))
        .map_err(|error| StarknetCommandError::ProviderError(error.into()))
}

fn build_transaction_status_response(status: &TransactionStatus) -> TransactionStatusResponse {
    match status {
        TransactionStatus::Received => TransactionStatusResponse {
            finality_status: FinalityStatus::Received,
            execution_status: None,
        },
        TransactionStatus::Rejected => TransactionStatusResponse {
            finality_status: FinalityStatus::Rejected,
            execution_status: None,
        },
        TransactionStatus::AcceptedOnL2(tx_exec_status) => TransactionStatusResponse {
            finality_status: FinalityStatus::AcceptedOnL2,
            execution_status: Some(build_execution_status(*tx_exec_status)),
        },
        TransactionStatus::AcceptedOnL1(tx_exec_status) => TransactionStatusResponse {
            finality_status: FinalityStatus::AcceptedOnL1,
            execution_status: Some(build_execution_status(*tx_exec_status)),
        },
    }
}

fn build_execution_status(status: TransactionExecutionStatus) -> ExecutionStatus {
    match status {
        TransactionExecutionStatus::Succeeded => ExecutionStatus::Succeeded,
        TransactionExecutionStatus::Reverted => ExecutionStatus::Reverted,
    }
}
