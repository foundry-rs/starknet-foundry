use super::CheatnetState;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::{
        execution::entry_point::execute_call_entry_point, panic_data::try_extract_panic_data,
    },
    common::create_execute_calldata,
};
use blockifier::execution::{
    call_info::CallInfo,
    entry_point::{CallEntryPoint, CallType, EntryPointExecutionResult},
    errors::{EntryPointExecutionError, PreExecutionError},
    syscalls::hint_processor::{SyscallCounter, SyscallHintProcessor},
};
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::format_next_item;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use conversions::{byte_array::ByteArray, IntoConv};
use serde::{Deserialize, Serialize};
use starknet_api::transaction::EventContent;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    deprecated_contract_class::EntryPointType,
};

#[derive(Clone, Debug, Default)]
pub struct UsedResources {
    pub syscall_counter: SyscallCounter,
    pub execution_resources: ExecutionResources,
    pub l2_to_l1_payload_lengths: Vec<usize>,
    pub l1_handler_payload_lengths: Vec<usize>,
    pub events: Vec<EventContent>,
}

/// Enum representing possible call execution result, along with the data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallResult {
    Success { ret_data: Vec<Felt252> },
    Failure(CallFailure),
}

/// Enum representing possible call failure and its' type.
/// `Panic` - Recoverable, meant to be caught by the user.
/// `Error` - Unrecoverable, equivalent of panic! in rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallFailure {
    Panic { panic_data: Vec<Felt252> },
    Error { msg: String },
}

pub enum AddressOrClassHash {
    ContractAddress(ContractAddress),
    ClassHash(ClassHash),
}

impl CallFailure {
    /// Maps blockifier-type error, to one that can be put into memory as panic-data (or re-raised)
    #[must_use]
    pub fn from_execution_error(
        err: &EntryPointExecutionError,
        starknet_identifier: &AddressOrClassHash,
    ) -> Self {
        match err {
            EntryPointExecutionError::ExecutionFailed { error_data } => {
                let err_data: Vec<Felt252> = error_data
                    .iter()
                    .map(|data| Felt252::from_bytes_be(data.bytes()))
                    .collect();

                // blockifier/src/execution_utils:274 (format_panic_data) (modified)
                let err_data_str = {
                    let mut felts = error_data.iter().map(|felt| (*felt).into_());
                    let mut items = Vec::new();
                    while let Some(item) = format_next_item(&mut felts) {
                        items.push(item.quote_if_string());
                    }
                    if let [item] = &items[..] {
                        item.clone()
                    } else {
                        items.join("\n")
                    }
                };

                for invalid_calldata_msg in [
                    "Failed to deserialize param #",
                    "Input too long for arguments",
                ] {
                    if err_data_str.contains(invalid_calldata_msg) {
                        return CallFailure::Error { msg: err_data_str };
                    }
                }

                CallFailure::Panic {
                    panic_data: err_data,
                }
            }
            EntryPointExecutionError::VirtualMachineExecutionErrorWithTrace { trace, .. } => {
                if let Some(panic_data) = try_extract_panic_data(trace) {
                    CallFailure::Panic { panic_data }
                } else {
                    CallFailure::Error { msg: trace.clone() }
                }
            }
            EntryPointExecutionError::PreExecutionError(PreExecutionError::EntryPointNotFound(
                selector,
            )) => {
                let selector_hash = selector.0;
                let msg = match starknet_identifier {
                    AddressOrClassHash::ContractAddress(address) => format!(
                        "Entry point selector {selector_hash} not found in contract {}",
                        address.0.key()
                    ),
                    AddressOrClassHash::ClassHash(class_hash) => format!(
                        "Entry point selector {selector_hash} not found for class hash {class_hash}"
                    ),
                };

                let panic_data_felts: Vec<Felt252> =
                    ByteArray::from(msg.as_str()).serialize_with_magic();

                CallFailure::Panic {
                    panic_data: panic_data_felts,
                }
            }
            EntryPointExecutionError::PreExecutionError(
                PreExecutionError::UninitializedStorageAddress(contract_address),
            ) => {
                let address = contract_address.0.key().to_string();
                let msg = format!("Contract not deployed at address: {address}");
                CallFailure::Error { msg }
            }
            EntryPointExecutionError::StateError(StateError::StateReadError(msg)) => {
                CallFailure::Error { msg: msg.clone() }
            }
            result => CallFailure::Error {
                msg: result.to_string(),
            },
        }
    }
}

impl CallResult {
    #[must_use]
    pub fn from_execution_result(
        result: &EntryPointExecutionResult<CallInfo>,
        starknet_identifier: &AddressOrClassHash,
    ) -> Self {
        match result {
            Ok(call_info) => Self::from_success(call_info),
            Err(err) => Self::from_err(err, starknet_identifier),
        }
    }

    #[must_use]
    pub fn from_success(call_info: &CallInfo) -> Self {
        let raw_return_data = &call_info.execution.retdata.0;

        let return_data = raw_return_data.iter().map(|data| (*data).into_()).collect();

        CallResult::Success {
            ret_data: return_data,
        }
    }

    #[must_use]
    pub fn from_err(
        err: &EntryPointExecutionError,
        starknet_identifier: &AddressOrClassHash,
    ) -> Self {
        CallResult::Failure(CallFailure::from_execution_error(err, starknet_identifier))
    }
}

pub fn call_l1_handler(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> CallResult {
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::L1Handler,
        entry_point_selector: entry_point_selector.clone().into_(),
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    call_entry_point(
        syscall_handler,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
    )
}

pub fn call_entry_point(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    mut entry_point: CallEntryPoint,
    starknet_identifier: &AddressOrClassHash,
) -> CallResult {
    let exec_result = execute_call_entry_point(
        &mut entry_point,
        syscall_handler.state,
        cheatnet_state,
        syscall_handler.resources,
        syscall_handler.context,
    );

    let result = CallResult::from_execution_result(&exec_result, starknet_identifier);

    if let Ok(call_info) = exec_result {
        syscall_handler.inner_calls.push(call_info);
    };

    result
}
