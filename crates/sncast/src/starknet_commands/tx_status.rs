use anyhow::{Context, Result};
use clap::Args;
use sncast::response::structs::{ExecutionStatus, FinalityStatus, TransactionStatusResponse};
use starknet::core::types::{FieldElement, TransactionExecutionStatus, TransactionStatus};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

#[derive(Args)]
#[command(about = "Get the status of a transaction")]
pub struct TxStatus {
    /// Hash of the transaction
    pub transaction_hash: FieldElement,
}

pub async fn tx_status(
    provider: &JsonRpcClient<HttpTransport>,
    transaction_hash: FieldElement,
) -> Result<TransactionStatusResponse> {
    provider
        .get_transaction_status(transaction_hash)
        .await
        .context("Failed to get transaction status")
        .map(|status| to_response(&status))
}

fn to_response(status: &TransactionStatus) -> TransactionStatusResponse {
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
            execution_status: Some(to_exec_status(*tx_exec_status)),
        },
        TransactionStatus::AcceptedOnL1(tx_exec_status) => TransactionStatusResponse {
            finality_status: FinalityStatus::AcceptedOnL1,
            execution_status: Some(to_exec_status(*tx_exec_status)),
        },
    }
}

fn to_exec_status(status: TransactionExecutionStatus) -> ExecutionStatus {
    match status {
        TransactionExecutionStatus::Succeeded => ExecutionStatus::Succeeded,
        TransactionExecutionStatus::Reverted => ExecutionStatus::Reverted,
    }
}
