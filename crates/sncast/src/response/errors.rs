use crate::{handle_rpc_error, ErrorData, WaitForTransactionError};
use anyhow;
use starknet::providers::ProviderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StarknetCommandError {
    #[error(transparent)]
    Unrecoverable(anyhow::Error),
    #[error(transparent)]
    Recoverable(#[from] RecoverableStarknetCommandError),
}

#[derive(Error, Debug)]
pub enum RecoverableStarknetCommandError {
    #[error("Failed to find {} artifact in starknet_artifacts.json file. Please make sure you have specified correct package using `--package` flag and that you have enabled sierra and casm code generation in Scarb.toml.", .0.data)]
    ContractArtifactsNotFound(ErrorData),
    #[error(transparent)]
    WaitForTransactionError(#[from] WaitForTransactionError),
    #[error(transparent)]
    ProviderError(ProviderError),
}

impl From<anyhow::Error> for StarknetCommandError {
    fn from(value: anyhow::Error) -> Self {
        StarknetCommandError::Unrecoverable(value)
    }
}

impl From<WaitForTransactionError> for StarknetCommandError {
    fn from(value: WaitForTransactionError) -> Self {
        StarknetCommandError::Recoverable(value.into())
    }
}

#[must_use]
pub fn handle_starknet_command_error(error: StarknetCommandError) -> anyhow::Error {
    match error {
        StarknetCommandError::Unrecoverable(error) => error,
        StarknetCommandError::Recoverable(error) => match error {
            RecoverableStarknetCommandError::ProviderError(err) => handle_rpc_error(err),
            _ => error.into(),
        },
    }
}
