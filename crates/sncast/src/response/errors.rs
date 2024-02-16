use crate::{handle_rpc_error, ErrorData, TransactionError, WaitForTransactionError};
use anyhow;
use starknet::providers::ProviderError;

#[derive(Debug)]
pub enum StarknetCommandError {
    Unrecoverable(anyhow::Error),
    Recoverable(RecoverableStarknetCommandError),
}

#[derive(Debug)]
pub enum RecoverableStarknetCommandError {
    ContractArtifactsNotFound(ErrorData),
    TimedOut,
    TransactionError(TransactionError),
    ProviderError(ProviderError),
}

impl From<anyhow::Error> for StarknetCommandError {
    fn from(value: anyhow::Error) -> Self {
        StarknetCommandError::Unrecoverable(value)
    }
}

impl From<WaitForTransactionError> for StarknetCommandError {
    fn from(value: WaitForTransactionError) -> Self {
        match value {
            WaitForTransactionError::TransactionError(error) => StarknetCommandError::Recoverable(
                RecoverableStarknetCommandError::TransactionError(error),
            ),
            WaitForTransactionError::TimedOut => {
                StarknetCommandError::Recoverable(RecoverableStarknetCommandError::TimedOut)
            }
            WaitForTransactionError::Other(error) => StarknetCommandError::Unrecoverable(error),
        }
    }
}

#[must_use]
pub fn handle_starknet_command_error(error: StarknetCommandError) -> anyhow::Error {
    match error {
        StarknetCommandError::Unrecoverable(error) => error,
        StarknetCommandError::Recoverable(error) => match error {
            RecoverableStarknetCommandError::ContractArtifactsNotFound(ErrorData { data: contract_name }) => anyhow::anyhow!("Failed to find {contract_name} artifact in starknet_artifacts.json file. Please make sure you have specified correct package using `--package` flag and that you have enabled sierra and casm code generation in Scarb.toml."),
            RecoverableStarknetCommandError::TransactionError(error) => match error {
                TransactionError::Rejected => anyhow::anyhow!("Transaction has been rejected"),
                TransactionError::Reverted(ErrorData { data: reason }) => anyhow::anyhow!("Transaction has been reverted = {reason}"),
            }
            RecoverableStarknetCommandError::ProviderError(err) => handle_rpc_error(err),
            RecoverableStarknetCommandError::TimedOut => anyhow::anyhow!("sncast timed out while waiting for transaction to succeed"),
        }
    }
}
