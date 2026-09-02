pub mod cairo1_execution;
pub mod calls;
pub mod cheated_syscalls;
pub mod deprecated;
pub mod entry_point;
pub mod execution_info;
pub mod execution_utils;
pub mod syscall_hooks;

#[cfg(feature = "starkloupe")]
use crate::runtime_extensions::outer_call_runtime_extension::execution::entry_point::EntryPointExecutionErrorWithTraceAndMemory;
#[cfg(feature = "starkloupe")]
use blockifier::execution::errors::EntryPointExecutionError;
#[cfg(feature = "starkloupe")]
use thiserror::Error;

#[cfg(feature = "starkloupe")]
#[derive(Debug, Error)]
pub enum StarkloupeEntryPointExecutionError {
    #[error(transparent)]
    EntryPointExecutionError(#[from] EntryPointExecutionError),
    #[error(transparent)]
    EntryPointExecutionErrorWithTraceAndMemory(#[from] EntryPointExecutionErrorWithTraceAndMemory),
}

#[cfg(feature = "starkloupe")]
pub type StarkloupeEntryPointExecutionResult<T> = Result<T, StarkloupeEntryPointExecutionError>;
