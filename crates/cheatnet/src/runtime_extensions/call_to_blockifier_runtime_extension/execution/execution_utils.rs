use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, CallResult,
};
use crate::runtime_extensions::common::sum_syscall_usage;
use crate::runtime_extensions::forge_runtime_extension::{
    get_nested_calls_syscalls_sierra_gas, get_nested_calls_syscalls_vm_resources,
};
use crate::state::{CheatedData, CheatnetState};
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::{CallEntryPoint, CallType, ExecutableCallEntryPoint};
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;

pub(crate) fn resolve_cheated_data_for_call(
    entry_point: &mut CallEntryPoint,
    cheatnet_state: &mut CheatnetState,
) -> CheatedData {
    if let CallType::Delegate = entry_point.call_type {
        // For delegate calls, we use the cheated data from the caller context.
        if cheatnet_state.trace_data.current_call_stack.size() == 1 {
            cheatnet_state.get_cheated_data(entry_point.caller_address)
        } else {
            cheatnet_state
                .trace_data
                .current_call_stack
                .top_cheated_data()
                .clone()
        }
    } else {
        let contract_address = entry_point.storage_address;
        let cheated_data = cheatnet_state.create_cheated_data(contract_address);
        cheatnet_state.update_cheats(&contract_address);
        cheated_data
    }
}

pub(crate) fn update_trace_data(
    call_info: &CallInfo,
    syscall_usage_vm_resources: &SyscallUsageMap,
    syscall_usage_sierra_gas: &SyscallUsageMap,
    cheatnet_state: &mut CheatnetState,
) {
    let nested_syscall_usage_vm_resources =
        get_nested_calls_syscalls_vm_resources(&cheatnet_state.trace_data.current_call_stack.top());
    let nested_syscall_usage_sierra_gas =
        get_nested_calls_syscalls_sierra_gas(&cheatnet_state.trace_data.current_call_stack.top());

    let syscall_usage_vm_resources = sum_syscall_usage(
        nested_syscall_usage_vm_resources,
        syscall_usage_vm_resources,
    );
    let syscall_usage_sierra_gas =
        sum_syscall_usage(nested_syscall_usage_sierra_gas, syscall_usage_sierra_gas);

    let signature = cheatnet_state
        .get_cheated_data(call_info.call.storage_address)
        .tx_info
        .signature
        .unwrap_or_default();

    cheatnet_state.trace_data.update_current_call(
        call_info.resources.clone(),
        call_info.execution.gas_consumed,
        syscall_usage_vm_resources,
        syscall_usage_sierra_gas,
        CallResult::from_non_error(call_info),
        &call_info.execution.l2_to_l1_messages,
        signature,
        call_info.execution.events.clone(),
    );
}

pub(crate) fn exit_error_call(
    error: &EntryPointExecutionError,
    cheatnet_state: &mut CheatnetState,
    entry_point: &ExecutableCallEntryPoint,
) {
    let identifier = match entry_point.call_type {
        CallType::Call => AddressOrClassHash::ContractAddress(entry_point.storage_address),
        CallType::Delegate => AddressOrClassHash::ClassHash(entry_point.class_hash),
    };
    let trace_data = &mut cheatnet_state.trace_data;

    // In case of a revert, clear all events and messages emitted by the current call.
    trace_data.clear_current_call_events_and_messages();

    trace_data.update_current_call_result(CallResult::from_err(error, &identifier));
    trace_data.exit_nested_call();
}
