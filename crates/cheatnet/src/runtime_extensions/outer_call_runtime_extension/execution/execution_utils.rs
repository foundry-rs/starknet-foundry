use crate::runtime_extensions::common::sum_syscall_usage;
use crate::runtime_extensions::forge_runtime_extension::{
    get_nested_calls_syscalls_sierra_gas, get_nested_calls_syscalls_vm_resources,
};
use crate::state::{CheatedData, CheatnetState};
use crate::trace_data::{from_error, from_non_error};
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::errors::AnnotatedEntryPointExecutionError;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;

pub(crate) fn resolve_cheated_data_for_call(
    entry_point: &mut CallEntryPoint,
    cheatnet_state: &mut CheatnetState,
) -> CheatedData {
    if let CallType::Delegate = entry_point.call_type {
        // When a delegate call is made directly by a test contract, `top_cheated_data()` will have a default value (all fields set to `None`).
        // Therefore, we need to get cheated data for the current contract, which is the test contract.
        if cheatnet_state.trace_data.current_call_stack.size() == 1 {
            cheatnet_state.get_cheated_data(entry_point.storage_address)
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
        from_non_error(call_info),
        &call_info.execution.l2_to_l1_messages,
        signature,
        call_info.execution.events.clone(),
    );
}

/// Clears `events` and `l2_to_l1_messages` from a reverted call and all its inner calls that did not fail.
/// This is part of `execute_inner_call` function in Blockifier.
/// Based on (blockifier 0.16.0-rc.1) <https://github.com/starkware-libs/sequencer/blob/050880eacde7f002f90267a1e7bebbbd40932578/crates/blockifier/src/execution/syscalls/syscall_base.rs#L441-L453>
pub(crate) fn clear_events_and_messages_from_reverted_call(reverted_call: &mut CallInfo) {
    let mut stack: Vec<&mut CallInfo> = vec![reverted_call];
    while let Some(call_info) = stack.pop() {
        call_info.execution.events.clear();
        call_info.execution.l2_to_l1_messages.clear();
        // Add inner calls that did not fail to the stack.
        // The events and l2_to_l1_messages of the failed calls were already cleared.
        stack.extend(
            call_info
                .inner_calls
                .iter_mut()
                .filter(|call_info| !call_info.execution.failed),
        );
    }
}

pub(crate) fn exit_error_call(
    error: &AnnotatedEntryPointExecutionError,
    cheatnet_state: &mut CheatnetState,
) {
    let trace_data = &mut cheatnet_state.trace_data;

    // In case of a revert, clear all events and messages emitted by the current call.
    trace_data.clear_current_call_events_and_messages();

    trace_data.update_current_call_result(from_error(error));
    trace_data.exit_nested_call();
}
