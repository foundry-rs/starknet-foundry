use crate::{ErrorData, WaitForTransactionError, handle_rpc_error};
use anyhow::anyhow;
use console::style;
use conversions::serde::serialize::CairoSerialize;

use conversions::byte_array::ByteArray;

use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};
use starknet::core::types::{ContractErrorData, StarknetError, TransactionExecutionErrorData};
use starknet::providers::ProviderError;
use thiserror::Error;

#[derive(Serialize, Debug)]
pub struct ResponseError {
    command: String,
    error: String,
}

impl ResponseError {
    #[must_use]
    pub fn new(command: String, error: String) -> Self {
        Self { command, error }
    }
}

impl Message for ResponseError {
    fn text(&self) -> String {
        format!(
            "Command: {}
{}: {}",
            self.command,
            style("Error").red(),
            self.error
        )
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

#[derive(Error, Debug, CairoSerialize)]
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

#[must_use]
pub fn handle_starknet_command_error(error: StarknetCommandError) -> anyhow::Error {
    match error {
        StarknetCommandError::ProviderError(err) => handle_rpc_error(err),
        _ => error.into(),
    }
}

#[derive(Debug, Error, CairoSerialize)]
pub enum SNCastProviderError {
    #[error(transparent)]
    StarknetError(SNCastStarknetError),
    #[error("Request rate limited")]
    RateLimited,
    #[error("Unknown RPC error: {0}")]
    UnknownError(#[from] anyhow::Error),
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

#[derive(Debug, Error, CairoSerialize)]
pub enum SNCastStarknetError {
    #[error("Node failed to receive transaction")]
    FailedToReceiveTransaction,
    #[error("There is no contract at the specified address")]
    ContractNotFound,
    #[error("Requested entrypoint does not exist in the contract")]
    EntryPointNotFound,
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
    #[error("The transaction's resources don't cover validation or the minimal transaction fee")]
    InsufficientResourcesForValidate,
    #[error("Account balance is too small to cover transaction fee")]
    InsufficientAccountBalance,
    #[error("Contract failed the validation = {0}")]
    ValidationFailure(ByteArray),
    #[error("Contract failed to compile in starknet")]
    CompilationFailed(ByteArray),
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
            StarknetError::ContractError(err) => SNCastStarknetError::ContractError(err),
            StarknetError::TransactionExecutionError(err) => {
                SNCastStarknetError::TransactionExecutionError(err)
            }
            StarknetError::ClassAlreadyDeclared => SNCastStarknetError::ClassAlreadyDeclared,
            StarknetError::InvalidTransactionNonce => SNCastStarknetError::InvalidTransactionNonce,
            StarknetError::InsufficientResourcesForValidate => {
                SNCastStarknetError::InsufficientResourcesForValidate
            }
            StarknetError::InsufficientAccountBalance => {
                SNCastStarknetError::InsufficientAccountBalance
            }
            StarknetError::ValidationFailure(err) => {
                SNCastStarknetError::ValidationFailure(ByteArray::from(err.as_str()))
            }
            StarknetError::CompilationFailed(msg) => {
                SNCastStarknetError::CompilationFailed(ByteArray::from(msg.as_str()))
            }
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
            StarknetError::EntrypointNotFound => SNCastStarknetError::EntryPointNotFound,
            other => SNCastStarknetError::UnexpectedError(anyhow!(other)),
        }
    }
}
