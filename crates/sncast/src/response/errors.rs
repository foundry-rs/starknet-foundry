use crate::{handle_rpc_error, ErrorData, WaitForTransactionError};
use anyhow::anyhow;
use cairo_felt::Felt252;
use conversions::felt252::SerializeAsFelt252Vec;
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
    ProviderError(#[from] SNCastProviderError),
}

impl SerializeAsFelt252Vec for StarknetCommandError {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            StarknetCommandError::UnknownError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(err.to_string().as_str().serialize_as_felt252_vec());
                res
            }
            StarknetCommandError::ContractArtifactsNotFound(err) => {
                let mut res = vec![Felt252::from(1)];
                res.extend(err.data.as_str().serialize_as_felt252_vec());
                res
            }
            StarknetCommandError::WaitForTransactionError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
            StarknetCommandError::ProviderError(err) => {
                let mut res = vec![Felt252::from(3)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
        }
    }
}

#[must_use]
pub fn handle_starknet_command_error(error: StarknetCommandError) -> anyhow::Error {
    match error {
        StarknetCommandError::ProviderError(err) => handle_rpc_error(err),
        _ => error.into(),
    }
}

#[derive(Debug, Error)]
pub enum SNCastProviderError {
    #[error(transparent)]
    StarknetError(SNCastStarknetError),
    #[error("Request rate limited")]
    RateLimited,
    #[error("Unknown RPC error: {0}")]
    UnknownError(#[from] anyhow::Error),
}

impl SerializeAsFelt252Vec for SNCastProviderError {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            SNCastProviderError::StarknetError(err) => {
                let mut res = vec![Felt252::from(0)];
                res.extend(err.serialize_as_felt252_vec());
                res
            }
            SNCastProviderError::RateLimited => {
                vec![Felt252::from(1)]
            }
            SNCastProviderError::UnknownError(err) => {
                let mut res = vec![Felt252::from(2)];
                res.extend(err.to_string().as_str().serialize_as_felt252_vec());
                res
            }
        }
    }
}

impl From<ProviderError> for SNCastProviderError {
    fn from(value: ProviderError) -> Self {
        match value {
            ProviderError::StarknetError(err) => SNCastProviderError::StarknetError(err.into()),
            ProviderError::RateLimited => SNCastProviderError::RateLimited,
            ProviderError::ArrayLengthMismatch => {
                SNCastProviderError::UnknownError(anyhow!("Array length mismatch"))
            }
            ProviderError::Other(err) => SNCastProviderError::UnknownError(anyhow!("{err}")),
        }
    }
}

#[derive(Debug, Error)]
pub enum SNCastStarknetError {
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

impl From<StarknetError> for SNCastStarknetError {
    fn from(value: StarknetError) -> Self {
        match value {
            StarknetError::FailedToReceiveTransaction => {
                SNCastStarknetError::FailedToReceiveTransaction
            }
            StarknetError::ContractNotFound => SNCastStarknetError::ContractNotFound,
            StarknetError::BlockNotFound => SNCastStarknetError::BlockNotFound,
            StarknetError::InvalidTransactionIndex => SNCastStarknetError::InvalidTransactionIndex,
            StarknetError::ClassHashNotFound => SNCastStarknetError::ClassHashNotFound,
            StarknetError::TransactionHashNotFound => SNCastStarknetError::TransactionHashNotFound,
            ContractError(err) => SNCastStarknetError::ContractError(err),
            TransactionExecutionError(err) => SNCastStarknetError::TransactionExecutionError(err),
            StarknetError::ClassAlreadyDeclared => SNCastStarknetError::ClassAlreadyDeclared,
            StarknetError::InvalidTransactionNonce => SNCastStarknetError::InvalidTransactionNonce,
            StarknetError::InsufficientMaxFee => SNCastStarknetError::InsufficientMaxFee,
            StarknetError::InsufficientAccountBalance => {
                SNCastStarknetError::InsufficientAccountBalance
            }
            ValidationFailure(err) => SNCastStarknetError::ValidationFailure(err),
            StarknetError::CompilationFailed => SNCastStarknetError::CompilationFailed,
            StarknetError::ContractClassSizeIsTooLarge => {
                SNCastStarknetError::ContractClassSizeIsTooLarge
            }
            StarknetError::NonAccount => SNCastStarknetError::NonAccount,
            StarknetError::DuplicateTx => SNCastStarknetError::DuplicateTx,
            StarknetError::CompiledClassHashMismatch => {
                SNCastStarknetError::CompiledClassHashMismatch
            }
            StarknetError::UnsupportedTxVersion => SNCastStarknetError::UnsupportedTxVersion,
            StarknetError::UnsupportedContractClassVersion => {
                SNCastStarknetError::UnsupportedContractClassVersion
            }
            StarknetError::UnexpectedError(err) => {
                SNCastStarknetError::UnexpectedError(anyhow!(err))
            }
            other => SNCastStarknetError::UnexpectedError(anyhow!(other)),
        }
    }
}

impl SerializeAsFelt252Vec for SNCastStarknetError {
    fn serialize_as_felt252_vec(&self) -> Vec<Felt252> {
        match self {
            SNCastStarknetError::FailedToReceiveTransaction => vec![Felt252::from(0)],
            SNCastStarknetError::ContractNotFound => vec![Felt252::from(1)],
            SNCastStarknetError::BlockNotFound => vec![Felt252::from(2)],
            SNCastStarknetError::InvalidTransactionIndex => vec![Felt252::from(3)],
            SNCastStarknetError::ClassHashNotFound => vec![Felt252::from(4)],
            SNCastStarknetError::TransactionHashNotFound => vec![Felt252::from(5)],
            SNCastStarknetError::ContractError(err) => {
                let mut res = vec![Felt252::from(6)];
                res.extend(err.revert_error.as_str().serialize_as_felt252_vec());
                res
            }
            SNCastStarknetError::TransactionExecutionError(err) => {
                let mut res = vec![Felt252::from(7), Felt252::from(err.transaction_index)];
                res.extend(err.execution_error.as_str().serialize_as_felt252_vec());
                res
            }
            SNCastStarknetError::ClassAlreadyDeclared => vec![Felt252::from(8)],
            SNCastStarknetError::InvalidTransactionNonce => vec![Felt252::from(9)],
            SNCastStarknetError::InsufficientMaxFee => vec![Felt252::from(10)],
            SNCastStarknetError::InsufficientAccountBalance => vec![Felt252::from(11)],
            SNCastStarknetError::ValidationFailure(err) => {
                let mut res = vec![Felt252::from(12)];
                res.extend(err.as_str().serialize_as_felt252_vec());
                res
            }
            SNCastStarknetError::CompilationFailed => vec![Felt252::from(13)],
            SNCastStarknetError::ContractClassSizeIsTooLarge => vec![Felt252::from(14)],
            SNCastStarknetError::NonAccount => vec![Felt252::from(15)],
            SNCastStarknetError::DuplicateTx => vec![Felt252::from(16)],
            SNCastStarknetError::CompiledClassHashMismatch => vec![Felt252::from(17)],
            SNCastStarknetError::UnsupportedTxVersion => vec![Felt252::from(18)],
            SNCastStarknetError::UnsupportedContractClassVersion => vec![Felt252::from(19)],
            SNCastStarknetError::UnexpectedError(err) => {
                let mut res = vec![Felt252::from(20)];
                res.extend(err.to_string().as_str().serialize_as_felt252_vec());
                res
            }
        }
    }
}
