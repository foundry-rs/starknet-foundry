use std::io;

use crate::rpc::CallContractFailure;
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use starknet_api::StarknetApiError;
use thiserror::Error;

pub mod declare;
pub mod deploy;
pub mod get_class_hash;
pub mod l1_handler_execute;
pub mod mock_call;
pub mod prank;
pub mod precalculate_address;
pub mod roll;
pub mod spoof;
pub mod spy_events;
pub mod warp;

// All errors that can be thrown from the hint executor have to be added here,
// to prevent the whole runner from panicking
#[derive(Error, Debug)]
pub enum EnhancedHintError {
    #[error(transparent)]
    Hint(#[from] HintError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    VirtualMachine(#[from] VirtualMachineError),
    #[error(transparent)]
    Memory(#[from] MemoryError),
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StarknetApi(#[from] StarknetApiError),
    #[error("Failed to parse {path} file")]
    FileParsing { path: String },
}

impl From<EnhancedHintError> for HintError {
    fn from(error: EnhancedHintError) -> Self {
        match error {
            EnhancedHintError::Hint(error) => error,
            error => HintError::CustomHint(error.to_string().into_boxed_str()),
        }
    }
}

/// A structure used for returning cheatcode errors in tests
#[derive(Debug)]
pub enum CheatcodeError {
    Recoverable(Vec<Felt252>),        // Return error result in cairo
    Unrecoverable(EnhancedHintError), // Fail whole test
}

impl From<EnhancedHintError> for CheatcodeError {
    fn from(error: EnhancedHintError) -> Self {
        CheatcodeError::Unrecoverable(error)
    }
}

impl From<CallContractFailure> for CheatcodeError {
    fn from(value: CallContractFailure) -> Self {
        match value {
            CallContractFailure::Panic { panic_data } => CheatcodeError::Recoverable(panic_data),
            CallContractFailure::Error { msg } => {
                CheatcodeError::Unrecoverable(HintError::CustomHint(Box::from(msg)).into())
            }
        }
    }
}
