use super::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ExecuteCallEntryPointExtraOptions, clear_handled_errors, execute_call_entry_point,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::execution_utils::clear_events_and_messages_from_reverted_call;
use crate::runtime_extensions::{
    call_to_blockifier_runtime_extension::panic_parser::try_extract_panic_data,
    common::create_execute_calldata,
};
use blockifier::execution::call_info::{CallExecution, ExecutionSummary, Retdata};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::syscalls::hint_processor::{
    ENTRYPOINT_FAILED_ERROR, SyscallExecutionError,
};
use blockifier::execution::syscalls::vm_syscall_utils::SyscallExecutorBaseError;
use blockifier::execution::{
    call_info::CallInfo,
    entry_point::CallType,
    errors::{EntryPointExecutionError, PreExecutionError},
    syscalls::hint_processor::SyscallHintProcessor,
};
use blockifier::execution::{
    entry_point::CallEntryPoint, syscalls::vm_syscall_utils::SyscallUsageMap,
};
use blockifier::state::errors::StateError;
use cairo_vm::vm::errors::hint_errors::HintError;
use conversions::{byte_array::ByteArray, serde::serialize::CairoSerialize, string::IntoHexStr};
use shared::utils::build_readable_text;
use starknet_api::core::EntryPointSelector;
use starknet_api::{
    contract_class::EntryPointType,
    core::{ClassHash, ContractAddress},
};
use starknet_types_core::felt::Felt;

#[derive(Clone, Debug, Default)]
pub struct UsedResources {
    pub syscall_usage: SyscallUsageMap,
    pub execution_summary: ExecutionSummary,
    pub l1_handler_payload_lengths: Vec<usize>,
}

#[derive(Debug, CairoSerialize)]
pub struct CallSuccess {
    pub ret_data: Vec<Felt>,
}

impl From<CallFailure> for SyscallExecutionError {
    fn from(value: CallFailure) -> Self {
        match value {
            CallFailure::Recoverable { panic_data } => Self::Revert {
                error_data: panic_data,
            },
            // TODO(#3307):
            // `SyscallExecutorBaseError::Hint` is chosen arbitrary to enable conversion by `try_extract_revert`
            // in `execute_syscall` function.
            // Ideally, we should pass the actual received error instead of a string.
            CallFailure::Unrecoverable { msg } => Self::SyscallExecutorBase(
                SyscallExecutorBaseError::Hint(HintError::CustomHint(Box::from(msg.to_string()))),
            ),
        }
    }
}

pub type CallResult = Result<CallSuccess, CallFailure>;

/// Enum representing a possible call failure and its type.
/// `Recoverable` - Meant to be caught by the user.
/// `Unrecoverable` - Equivalent of `panic!` in rust.
#[derive(Debug, Clone, CairoSerialize)]
pub enum CallFailure {
    Recoverable { panic_data: Vec<Felt> },
    Unrecoverable { msg: ByteArray },
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
                    CallFailure::Unrecoverable {
                        msg: ByteArray::from(err_data_str.as_str()),
                    }
                } else {
                    CallFailure::Recoverable {
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

                CallFailure::Recoverable {
                    panic_data: panic_data_felts,
                }
            }
            EntryPointExecutionError::PreExecutionError(
                PreExecutionError::UninitializedStorageAddress(contract_address),
            ) => {
                let address_str = contract_address.into_hex_string();
                let msg = format!("Contract not deployed at address: {address_str}");

                let panic_data_felts = ByteArray::from(msg.as_str()).serialize_with_magic();

                CallFailure::Recoverable {
                    panic_data: panic_data_felts,
                }
            }
            EntryPointExecutionError::StateError(StateError::StateReadError(msg)) => {
                CallFailure::Unrecoverable {
                    msg: ByteArray::from(msg.as_str()),
                }
            }
            error => {
                let error_string = error.to_string();
                if let Some(panic_data) = try_extract_panic_data(&error_string) {
                    CallFailure::Recoverable { panic_data }
                } else {
                    CallFailure::Unrecoverable {
                        msg: ByteArray::from(error_string.as_str()),
                    }
                }
            }
        }
    }
}

pub fn from_non_error(call_info: &CallInfo) -> Result<CallSuccess, CallFailure> {
    let return_data = &call_info.execution.retdata.0;

    if call_info.execution.failed {
        return Err(CallFailure::Recoverable {
            panic_data: return_data.clone(),
        });
    }

    Ok(CallSuccess {
        ret_data: return_data.clone(),
    })
}

pub fn from_error(
    err: &EntryPointExecutionError,
    starknet_identifier: &AddressOrClassHash,
) -> Result<CallSuccess, CallFailure> {
    Err(CallFailure::from_execution_error(err, starknet_identifier))
}

pub fn call_l1_handler(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
) -> Result<CallSuccess, CallFailure> {
    let calldata = create_execute_calldata(calldata);
    let mut remaining_gas = i64::MAX as u64;
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::L1Handler,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: remaining_gas,
    };

    call_entry_point(
        syscall_handler,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
        &mut remaining_gas,
    )
}

pub fn call_entry_point(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    mut entry_point: CallEntryPoint,
    starknet_identifier: &AddressOrClassHash,
    remaining_gas: &mut u64,
) -> Result<CallSuccess, CallFailure> {
    let revert_idx = syscall_handler.base.context.revert_infos.0.len();
    let result = execute_call_entry_point(
        &mut entry_point,
        syscall_handler.base.state,
        cheatnet_state,
        syscall_handler.base.context,
        remaining_gas,
        &ExecuteCallEntryPointExtraOptions {
            trace_data_handled_by_revert_call: false,
        },
    )
    .map_err(|err| CallFailure::from_execution_error(&err, starknet_identifier));

    let call_info = match result {
        Ok(call_info) => call_info,
        Err(CallFailure::Recoverable { panic_data }) => {
            let storage_class_hash = syscall_handler
                .base
                .state
                .get_class_hash_at(entry_point.storage_address)
                .expect("There should be a class hash at the storage address");
            let maybe_replacement_class = cheatnet_state
                .replaced_bytecode_contracts
                .get(&entry_point.storage_address)
                .copied();
            let class_hash = entry_point
                .class_hash
                .or(maybe_replacement_class)
                .unwrap_or(storage_class_hash);

            let current_tracked_resource = syscall_handler
                .base
                .state
                .get_compiled_class(class_hash)
                .map(|compiled_class| {
                    compiled_class.get_current_tracked_resource(syscall_handler.base.context)
                })
                .unwrap_or(TrackedResource::SierraGas);

            CallInfo {
                call: entry_point.into_executable(class_hash).into(),
                execution: CallExecution {
                    retdata: Retdata(panic_data.clone()),
                    failed: true,
                    gas_consumed: 0,
                    ..CallExecution::default()
                },
                tracked_resource: current_tracked_resource,
                ..CallInfo::default()
            }
        }
        Err(err) => {
            return Err(err);
        }
    };

    let mut raw_retdata = call_info.execution.retdata.0.clone();
    let failed = call_info.execution.failed;
    syscall_handler.base.inner_calls.push(call_info.clone());

    if failed {
        clear_handled_errors(&call_info, cheatnet_state);

        syscall_handler
            .base
            .context
            .revert(revert_idx, syscall_handler.base.state)
            .expect("Failed to revert state");

        // Delete events and l2_to_l1_messages from the reverted call.
        let reverted_call = syscall_handler.base.inner_calls.last_mut().unwrap();
        clear_events_and_messages_from_reverted_call(reverted_call);

        raw_retdata.push(Felt::from_hex(ENTRYPOINT_FAILED_ERROR).expect("Conversion should work"));
        return Err(CallFailure::Recoverable {
            panic_data: raw_retdata,
        });
    }

    Ok(CallSuccess {
        ret_data: raw_retdata,
    })
}
