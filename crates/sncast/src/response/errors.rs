use crate::{handle_rpc_error, ErrorData, WaitForTransactionError};
use anyhow::anyhow;
use starknet::core::types::StarknetError::{
    ContractError, TransactionExecutionError, ValidationFailure,
};
use starknet::core::types::{ContractErrorData, StarknetError, TransactionExecutionErrorData};
use starknet::providers::ProviderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StarknetCommandError {
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
    #[error("Failed to find {} artifact in starknet_artifacts.json file. Please make sure you have specified correct package using `--package` flag and that you have enabled sierra and casm code generation in Scarb.toml.", .0.data)]
    ContractArtifactsNotFound(ErrorData),
    #[error(transparent)]
    WaitForTransactionError(#[from] WaitForTransactionError),
    #[error(transparent)]
    ProviderError(#[from] RestrictedProviderError),
}

#[must_use]
pub fn handle_starknet_command_error(error: StarknetCommandError) -> anyhow::Error {
    match error {
        StarknetCommandError::ProviderError(err) => handle_rpc_error(err),
        _ => error.into(),
    }
}

#[derive(Debug, Error)]
pub enum RestrictedProviderError {
    #[error(transparent)]
    StarknetError(RestrictedStarknetError),
    #[error("Request rate limited")]
    RateLimited,
    #[error("Unknown RPC error: {0}")]
    UnknownError(#[from] anyhow::Error),
}

impl From<ProviderError> for RestrictedProviderError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::StarknetError(err) => RestrictedProviderError::StarknetError(err.into()),
            ProviderError::RateLimited => RestrictedProviderError::RateLimited,
            ProviderError::ArrayLengthMismatch => {
                RestrictedProviderError::UnknownError(anyhow!("Array length mismatch"))
            }
            ProviderError::Other(err) => RestrictedProviderError::UnknownError(anyhow!("{err}")),
        }
    }
}

#[derive(Debug, Error)]
pub enum RestrictedStarknetError {
    #[error("Node failed to receive transaction")]
    FailedToReceiveTransaction,
    #[error("There is no contract at the specified address")]
    ContractNotFound,
    #[error("Block was not found")]
    BlockNotFound,
    #[error("There is no transaction with such an index")]
    InvalidTransactionIndex,
    #[error("Provided class hash does not exist")]
    ClassHashNotFound,
    #[error("Transaction with provided hash was not found (does not exist)")]
    TransactionHashNotFound,
    #[error("An error occurred in the called contract = {0:?}")]
    ContractError(ContractErrorData),
    #[error("Transaction execution error = {0:?}")]
    TransactionExecutionError(TransactionExecutionErrorData),
    #[error("Contract with the same class hash is already declared")]
    ClassAlreadyDeclared,
    #[error("Invalid transaction nonce")]
    InvalidTransactionNonce,
    #[error("Max fee is smaller than the minimal transaction cost")]
    InsufficientMaxFee,
    #[error("Account balance is too small to cover transaction fee")]
    InsufficientAccountBalance,
    #[error("Contract failed the validation = {0}")]
    ValidationFailure(String),
    #[error("Contract failed to compile in starknet")]
    CompilationFailed,
    #[error("Contract class size is too large")]
    ContractClassSizeIsTooLarge,
    #[error("No account")]
    NonAccount,
    #[error("Transaction already exists")]
    DuplicateTx,
    #[error("Compiled class hash mismatch")]
    CompiledClassHashMismatch,
    #[error("Unsupported transaction version")]
    UnsupportedTxVersion,
    #[error("Unsupported contract class version")]
    UnsupportedContractClassVersion,
    #[error("Unexpected RPC error occurred: {0}")]
    UnexpectedError(anyhow::Error),
}

impl From<StarknetError> for RestrictedStarknetError {
    fn from(value: StarknetError) -> Self {
        match value {
            StarknetError::FailedToReceiveTransaction => {
                RestrictedStarknetError::FailedToReceiveTransaction
            }
            StarknetError::ContractNotFound => RestrictedStarknetError::ContractNotFound,
            StarknetError::BlockNotFound => RestrictedStarknetError::BlockNotFound,
            StarknetError::InvalidTransactionIndex => {
                RestrictedStarknetError::InvalidTransactionIndex
            }
            StarknetError::ClassHashNotFound => RestrictedStarknetError::ClassHashNotFound,
            StarknetError::TransactionHashNotFound => {
                RestrictedStarknetError::TransactionHashNotFound
            }
            ContractError(err) => RestrictedStarknetError::ContractError(err),
            TransactionExecutionError(err) => {
                RestrictedStarknetError::TransactionExecutionError(err)
            }
            StarknetError::ClassAlreadyDeclared => RestrictedStarknetError::ClassAlreadyDeclared,
            StarknetError::InvalidTransactionNonce => {
                RestrictedStarknetError::InvalidTransactionNonce
            }
            StarknetError::InsufficientMaxFee => RestrictedStarknetError::InsufficientMaxFee,
            StarknetError::InsufficientAccountBalance => {
                RestrictedStarknetError::InsufficientAccountBalance
            }
            ValidationFailure(err) => RestrictedStarknetError::ValidationFailure(err),
            StarknetError::CompilationFailed => RestrictedStarknetError::CompilationFailed,
            StarknetError::ContractClassSizeIsTooLarge => {
                RestrictedStarknetError::ContractClassSizeIsTooLarge
            }
            StarknetError::NonAccount => RestrictedStarknetError::NonAccount,
            StarknetError::DuplicateTx => RestrictedStarknetError::DuplicateTx,
            StarknetError::CompiledClassHashMismatch => {
                RestrictedStarknetError::CompiledClassHashMismatch
            }
            StarknetError::UnsupportedTxVersion => RestrictedStarknetError::UnsupportedTxVersion,
            StarknetError::UnsupportedContractClassVersion => {
                RestrictedStarknetError::UnsupportedContractClassVersion
            }
            StarknetError::UnexpectedError(err) => {
                RestrictedStarknetError::UnexpectedError(anyhow!(err))
            }
            other => RestrictedStarknetError::UnexpectedError(anyhow!(other)),
        }
    }
}
