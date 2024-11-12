use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallFailure;
use cairo_vm::vm::errors::hint_errors::HintError;
use runtime::EnhancedHintError;
use starknet_types_core::felt::Felt;

pub mod cheat_block_number;
pub mod cheat_block_timestamp;
pub mod cheat_caller_address;
pub mod cheat_execution_info;
pub mod cheat_sequencer_address;
pub mod declare;
pub mod deploy;
pub mod generate_random_felt;
pub mod get_class_hash;
pub mod l1_handler_execute;
pub mod mock_call;
pub mod precalculate_address;
pub mod replace_bytecode;
pub mod spy_events;
pub mod spy_messages_to_l1;
pub mod storage;

/// A structure used for returning cheatcode errors in tests
#[derive(Debug)]
pub enum CheatcodeError {
    Recoverable(Vec<Felt>),           // Return error result in cairo
    Unrecoverable(EnhancedHintError), // Fail whole test
}

impl From<EnhancedHintError> for CheatcodeError {
    fn from(error: EnhancedHintError) -> Self {
        CheatcodeError::Unrecoverable(error)
    }
}

impl From<CallFailure> for CheatcodeError {
    fn from(value: CallFailure) -> Self {
        match value {
            CallFailure::Panic { panic_data } => CheatcodeError::Recoverable(panic_data),
            CallFailure::Error { msg } => CheatcodeError::Unrecoverable(
                HintError::CustomHint(Box::from(msg.to_string())).into(),
            ),
        }
    }
}
