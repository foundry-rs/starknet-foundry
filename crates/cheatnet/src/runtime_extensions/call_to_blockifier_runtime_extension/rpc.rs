use super::CheatnetState;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::{
        execution::entry_point::execute_call_entry_point, panic_data::try_extract_panic_data,
    },
    common::create_execute_calldata,
};
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::execution::{
    call_info::CallInfo,
    entry_point::{CallType, EntryPointExecutionResult},
    errors::{EntryPointExecutionError, PreExecutionError},
    syscalls::hint_processor::{SyscallHintProcessor, SyscallUsageMap},
};
use blockifier::state::errors::StateError;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use conversions::{byte_array::ByteArray, serde::serialize::CairoSerialize, string::IntoHexStr};
use shared::utils::build_readable_text;
use starknet_api::execution_resources::GasAmount;
use starknet_api::{
    contract_class::EntryPointType,
    core::{ClassHash, ContractAddress},
};
use starknet_api::{core::EntryPointSelector, transaction::EventContent};
use starknet_types_core::felt::Felt;

#[derive(Clone, Debug, Default)]
pub struct UsedResources {
    pub syscall_usage: SyscallUsageMap,
    pub execution_resources: ExecutionResources,
    pub gas_consumed: GasAmount,
    pub l2_to_l1_payload_lengths: Vec<usize>,
    pub l1_handler_payload_lengths: Vec<usize>,
    pub events: Vec<EventContent>,
}

/// Enum representing possible call execution result, along with the data
#[derive(Debug, Clone, CairoSerialize)]
pub enum CallResult {
    Success { ret_data: Vec<Felt> },
    Failure(CallFailure),
}

/// Enum representing possible call failure and its type.
/// `Panic` - Recoverable, meant to be caught by the user.
/// `Error` - Unrecoverable, equivalent of panic! in rust.
#[derive(Debug, Clone, CairoSerialize)]
pub enum CallFailure {
    Panic { panic_data: Vec<Felt> },
    Error { msg: ByteArray },
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
            EntryPointExecutionError::ExecutionFailed { error_trace } => {
                let err_data = error_trace.last_retdata.clone().0;

                let err_data_str = build_readable_text(err_data.as_slice()).unwrap_or_default();

                if err_data_str.contains("Failed to deserialize param #")
                    || err_data_str.contains("Input too long for arguments")
                {
                    CallFailure::Error {
                        msg: ByteArray::from(err_data_str.as_str()),
                    }
                } else {
                    CallFailure::Panic {
                        panic_data: err_data,
                    }
                }
            }
            EntryPointExecutionError::PreExecutionError(PreExecutionError::EntryPointNotFound(
                selector,
            )) => {
                let selector_hash = selector.into_hex_string();
                let msg = match starknet_identifier {
                    AddressOrClassHash::ContractAddress(address) => format!(
                        "Entry point selector {selector_hash} not found in contract {}",
                        address.into_hex_string()
                    ),
                    AddressOrClassHash::ClassHash(class_hash) => format!(
                        "Entry point selector {selector_hash} not found for class hash {}",
                        class_hash.into_hex_string()
                    ),
                };

                let panic_data_felts = ByteArray::from(msg.as_str()).serialize_with_magic();

                CallFailure::Panic {
                    panic_data: panic_data_felts,
                }
            }
            EntryPointExecutionError::PreExecutionError(
                PreExecutionError::UninitializedStorageAddress(contract_address),
            ) => {
                let address_str = contract_address.into_hex_string();
                let msg = format!("Contract not deployed at address: {address_str}");

                let panic_data_felts = ByteArray::from(msg.as_str()).serialize_with_magic();

                CallFailure::Panic {
                    panic_data: panic_data_felts,
                }
            }
            EntryPointExecutionError::StateError(StateError::StateReadError(msg)) => {
                CallFailure::Error {
                    msg: ByteArray::from(msg.as_str()),
                }
            }
            error => {
                let error_string = error.to_string();
                if let Some(panic_data) = try_extract_panic_data(&error_string) {
                    CallFailure::Panic { panic_data }
                } else {
                    CallFailure::Error {
                        msg: ByteArray::from(error_string.as_str()),
                    }
                }
            }
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
            Ok(call_info) => Self::from_non_error(call_info),
            Err(err) => Self::from_err(err, starknet_identifier),
        }
    }

    #[must_use]
    pub fn from_non_error(call_info: &CallInfo) -> Self {
        let return_data = &call_info.execution.retdata.0;

        if call_info.execution.failed {
            return CallResult::Failure(CallFailure::Panic {
                panic_data: return_data.clone(),
            });
        }

        CallResult::Success {
            ret_data: return_data.clone(),
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
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
) -> CallResult {
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::L1Handler,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: i64::MAX as u64,
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
        syscall_handler.base.state,
        cheatnet_state,
        syscall_handler.base.context,
        false,
    );

    let result = CallResult::from_execution_result(&exec_result, starknet_identifier);

    if let Ok(call_info) = exec_result {
        syscall_handler.base.inner_calls.push(call_info);
    }

    result
}
