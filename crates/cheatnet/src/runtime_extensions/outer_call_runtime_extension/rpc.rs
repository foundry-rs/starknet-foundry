use super::CheatnetState;
use crate::runtime_extensions::common::create_execute_calldata;
use crate::runtime_extensions::outer_call_runtime_extension::execution::entry_point::{
    ExecuteCallEntryPointExtraOptions, clear_handled_errors, execute_call_entry_point,
};
use crate::runtime_extensions::outer_call_runtime_extension::execution::execution_utils::clear_events_and_messages_from_reverted_call;
use blockifier::execution::call_info::{CallExecution, ExecutionSummary, Retdata};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::syscalls::hint_processor::{
    ENTRYPOINT_FAILED_ERROR_FELT, SyscallExecutionError,
};
use blockifier::execution::{
    call_info::CallInfo,
    entry_point::CallType,
    errors::{AnnotatedEntryPointExecutionError, EntryPointExecutionError, PreExecutionError},
    syscalls::hint_processor::SyscallHintProcessor,
};
use blockifier::execution::{
    entry_point::CallEntryPoint, syscalls::vm_syscall_utils::SyscallUsageMap,
};
use conversions::{byte_array::ByteArray, serde::serialize::CairoSerialize, string::IntoHexStr};
use starknet_api::core::EntryPointSelector;
use starknet_api::{contract_class::EntryPointType, core::ContractAddress};
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
            CallFailure::Unrecoverable(error) => error.into(),
        }
    }
}

/// Result of a contract call as returned by [`call_entry_point`] / [`call_l1_handler`].
///
/// Unlike [`CallTraceResult`](crate::trace_data::CallTraceResult), this keeps the original
/// [`EntryPointExecutionError`] for the unrecoverable case, so it can be propagated without being
/// reduced to a string.
pub type CallEntryPointResult = Result<CallSuccess, CallFailure>;

/// Call failure returned by [`call_entry_point`] / [`call_l1_handler`].
/// `Recoverable` - Meant to be caught by the user.
/// `Unrecoverable` - Equivalent of `panic!` in rust.
#[derive(Debug)]
pub enum CallFailure {
    Recoverable { panic_data: Vec<Felt> },
    Unrecoverable(AnnotatedEntryPointExecutionError),
}

/// Classifies a blockifier error as recoverable (catchable by the user), returning the panic
/// data to put into memory, or `None` if the error is unrecoverable.
pub(crate) fn recoverable_panic_data(err: &AnnotatedEntryPointExecutionError) -> Option<Vec<Felt>> {
    match err.unannotated() {
        EntryPointExecutionError::PreExecutionError(
            PreExecutionError::UninitializedStorageAddress(contract_address),
        ) => {
            let address_str = contract_address.into_hex_string();
            let msg = format!("Contract not deployed at address: {address_str}");

            Some(ByteArray::from(msg.as_str()).serialize_with_magic())
        }
        _ => None,
    }
}

impl CallFailure {
    /// Maps a blockifier error to a call failure, keeping the original error when unrecoverable.
    #[must_use]
    pub fn from_execution_error(err: AnnotatedEntryPointExecutionError) -> Self {
        match recoverable_panic_data(&err) {
            Some(panic_data) => CallFailure::Recoverable { panic_data },
            None => CallFailure::Unrecoverable(err),
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

pub fn call_l1_handler(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
) -> CallEntryPointResult {
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
        &mut remaining_gas,
    )
}

pub fn call_entry_point(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    mut entry_point: CallEntryPoint,
    remaining_gas: &mut u64,
) -> CallEntryPointResult {
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
    .map_err(CallFailure::from_execution_error);

    let call_info = match result {
        Ok(call_info) => call_info,
        Err(CallFailure::Recoverable { panic_data }) => {
            build_failed_call_info(syscall_handler, cheatnet_state, entry_point, &panic_data)
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

        raw_retdata.push(ENTRYPOINT_FAILED_ERROR_FELT);
        return Err(CallFailure::Recoverable {
            panic_data: raw_retdata,
        });
    }

    Ok(CallSuccess {
        ret_data: raw_retdata,
    })
}

fn build_failed_call_info(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &CheatnetState,
    entry_point: CallEntryPoint,
    panic_data: &[Felt],
) -> CallInfo {
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
        .map_or(TrackedResource::SierraGas, |compiled_class| {
            compiled_class.get_current_tracked_resource(syscall_handler.base.context)
        });

    CallInfo {
        call: entry_point.into_executable(class_hash).into(),
        execution: CallExecution {
            retdata: Retdata(panic_data.to_vec()),
            failed: true,
            ..CallExecution::default()
        },
        tracked_resource: current_tracked_resource,
        ..CallInfo::default()
    }
}
