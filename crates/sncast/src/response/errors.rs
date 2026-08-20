use crate::{ErrorData, WaitForTransactionError, handle_rpc_error};
use anyhow::anyhow;
use console::style;
use conversions::padded_felt::PaddedFelt;

use conversions::byte_array::ByteArray;

use foundry_ui::Message;
use serde_json::{Value, json};
use starknet_rust::core::types::{ContractErrorData, StarknetError, TransactionExecutionErrorData};
use starknet_rust::providers::ProviderError;
use thiserror::Error;

#[derive(Debug)]
pub struct ResponseError {
    command: String,
    error: String,
    flat_error: String,
}

impl ResponseError {
    #[must_use]
    pub fn new(command: String, error: String) -> Self {
        Self {
            command,
            error: error.clone(),
            flat_error: error,
        }
    }

    #[must_use]
    pub fn from_anyhow(command: String, error: &anyhow::Error) -> Self {
        Self {
            command,
            error: format!("{error:?}"),
            flat_error: format!("{error:#}"),
        }
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
        json!({
            "error": self.flat_error,
        })
    }
}

#[derive(Error, Debug)]
pub enum StarknetCommandError {
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
    #[error("Failed to find {} artifact in starknet_artifacts.json file. Please make sure you have specified correct package using `--package` flag.", .0.data)]
    ContractArtifactsNotFound(ErrorData),
    #[error(transparent)]
    WaitForTransactionError(#[from] WaitForTransactionError),
    #[error(transparent)]
    ProviderError(#[from] SNCastProviderError),
    #[error("{}", .0.data)]
    ContractResolutionError(ErrorData),
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

fn format_validation_failure(address: Option<&PaddedFelt>, message: &ByteArray) -> String {
    match address {
        Some(addr) => format!("Account {addr:#x} failed the validation = {message}"),
        None => format!("Contract failed the validation = {message}"),
    }
}

fn format_insufficient_account_balance(address: Option<&PaddedFelt>) -> String {
    match address {
        Some(addr) => format!("Account {addr:#x} balance is too small to cover transaction fee"),
        None => "Account balance is too small to cover transaction fee".to_string(),
    }
}

#[derive(Debug, Error)]
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
    #[error("Contract with class hash {0:#x} is already declared")]
    ClassAlreadyDeclared(PaddedFelt),
    #[error("Invalid transaction nonce")]
    InvalidTransactionNonce,
    #[error("The transaction's resources don't cover validation or the minimal transaction fee")]
    InsufficientResourcesForValidate,
    #[error("{}", format_insufficient_account_balance(.address.as_ref()))]
    InsufficientAccountBalance { address: Option<PaddedFelt> },
    #[error("{}", format_validation_failure(.address.as_ref(), .message))]
    ValidationFailure {
        address: Option<PaddedFelt>,
        message: ByteArray,
    },
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
            StarknetError::ClassAlreadyDeclared => {
                unreachable!(
                    "ClassAlreadyDeclared error requires class hash parameter which is present in StarknetError::ClassAlreadyDeclared. This conversion should not be used."
                )
            }
            StarknetError::InvalidTransactionNonce(_) => {
                SNCastStarknetError::InvalidTransactionNonce
            }
            StarknetError::InsufficientResourcesForValidate => {
                SNCastStarknetError::InsufficientResourcesForValidate
            }
            StarknetError::InsufficientAccountBalance => {
                SNCastStarknetError::InsufficientAccountBalance { address: None }
            }
            StarknetError::ValidationFailure(err) => SNCastStarknetError::ValidationFailure {
                address: None,
                message: ByteArray::from(err.as_str()),
            },
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

impl SNCastStarknetError {
    /// Same as [`From<StarknetError>`], but attaches the account address to the errors that are
    /// caused by a specific account, so the user knows which account needs fixing.
    #[must_use]
    pub fn from_starknet_error_with_account(value: StarknetError, address: PaddedFelt) -> Self {
        match value {
            StarknetError::ValidationFailure(err) => SNCastStarknetError::ValidationFailure {
                address: Some(address),
                message: ByteArray::from(err.as_str()),
            },
            StarknetError::InsufficientAccountBalance => {
                SNCastStarknetError::InsufficientAccountBalance {
                    address: Some(address),
                }
            }
            other => SNCastStarknetError::from(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PaddedFelt, SNCastStarknetError, StarknetError};
    use conversions::byte_array::ByteArray;
    use starknet_types_core::felt::Felt;

    #[test]
    fn validation_failure_message_includes_account_address() {
        let error = SNCastStarknetError::ValidationFailure {
            address: Some(PaddedFelt(Felt::from_hex_unchecked("0x123"))),
            message: ByteArray::from("insufficient balance"),
        };

        assert_eq!(
            error.to_string(),
            "Account 0x0000000000000000000000000000000000000000000000000000000000000123 failed the validation = insufficient balance"
        );
    }

    #[test]
    fn validation_failure_message_without_address_keeps_legacy_text() {
        let error = SNCastStarknetError::ValidationFailure {
            address: None,
            message: ByteArray::from("insufficient balance"),
        };

        assert_eq!(
            error.to_string(),
            "Contract failed the validation = insufficient balance"
        );
    }

    #[test]
    fn insufficient_account_balance_message_includes_account_address() {
        let error = SNCastStarknetError::InsufficientAccountBalance {
            address: Some(PaddedFelt(Felt::from_hex_unchecked("0x123"))),
        };

        assert_eq!(
            error.to_string(),
            "Account 0x0000000000000000000000000000000000000000000000000000000000000123 balance is too small to cover transaction fee"
        );
    }

    #[test]
    fn insufficient_account_balance_message_without_address_keeps_legacy_text() {
        let error = SNCastStarknetError::InsufficientAccountBalance { address: None };

        assert_eq!(
            error.to_string(),
            "Account balance is too small to cover transaction fee"
        );
    }

    #[test]
    fn from_starknet_error_with_account_attaches_address() {
        let error = SNCastStarknetError::from_starknet_error_with_account(
            StarknetError::InsufficientAccountBalance,
            PaddedFelt(Felt::from_hex_unchecked("0x123")),
        );

        assert!(matches!(
            error,
            SNCastStarknetError::InsufficientAccountBalance { address: Some(_) }
        ));
    }

    #[test]
    fn from_starknet_error_with_account_passes_through_unrelated_errors() {
        let error = SNCastStarknetError::from_starknet_error_with_account(
            StarknetError::ContractNotFound,
            PaddedFelt(Felt::from_hex_unchecked("0x123")),
        );

        assert!(matches!(error, SNCastStarknetError::ContractNotFound));
    }
}
