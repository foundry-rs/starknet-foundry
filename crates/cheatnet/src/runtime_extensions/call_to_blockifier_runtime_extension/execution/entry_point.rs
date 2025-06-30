use super::cairo1_execution::execute_entry_point_call_cairo1;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::deprecated::cairo0_execution::execute_entry_point_call_cairo0;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{AddressOrClassHash, CallResult};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::common::{get_relocated_vm_trace, get_syscalls_gas_consumed, sum_syscall_usage};
use crate::state::{CallTrace, CallTraceNode, CheatStatus};
use blockifier::execution::call_info::{CallExecution, Retdata, StorageAccessTracker};
use blockifier::execution::contract_class::{RunnableCompiledClass, TrackedResource};

use blockifier::execution::entry_point::{EntryPointRevertInfo, ExecutableCallEntryPoint};
use blockifier::execution::stack_trace::{
    Cairo1RevertHeader, extract_trailing_cairo1_revert_trace,
};
use blockifier::execution::syscalls::hint_processor::{
    ENTRYPOINT_NOT_FOUND_ERROR, OUT_OF_GAS_ERROR,
};
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use blockifier::{
    execution::{
        call_info::CallInfo,
        entry_point::{
            CallEntryPoint, CallType, ConstructorContext, EntryPointExecutionContext,
            EntryPointExecutionResult, FAULTY_CLASS_HASH, handle_empty_constructor,
        },
        errors::{EntryPointExecutionError, PreExecutionError},
    },
    state::state_api::State,
};
use cairo_vm::vm::runners::cairo_runner::{CairoRunner, ExecutionResources};
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use conversions::FromConv;
use conversions::string::TryFromHexStr;
use shared::vm::VirtualMachineExt;
use starknet_api::{
    contract_class::EntryPointType,
    core::ClassHash,
    transaction::{TransactionVersion, fields::Calldata},
};
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use thiserror::Error;

pub(crate) type ContractClassEntryPointExecutionResult =
    Result<CallInfoWithExecutionData, EntryPointExecutionErrorWithTrace>;

pub(crate) struct CallInfoWithExecutionData {
    pub call_info: CallInfo,
    pub syscall_usage: SyscallUsageMap,
    pub vm_trace: Option<Vec<RelocatedTraceEntry>>,
}

// blockifier/src/execution/entry_point.rs:180 (CallEntryPoint::execute)
#[expect(clippy::too_many_lines)]
pub fn execute_call_entry_point(
    entry_point: &mut CallEntryPoint, // Instead of 'self'
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    is_revertable: bool,
) -> EntryPointExecutionResult<CallInfo> {
    let cheated_data = if let CallType::Delegate = entry_point.call_type {
        cheatnet_state
            .trace_data
            .current_call_stack
            .top_cheated_data()
            .clone()
    } else {
        let contract_address = entry_point.storage_address;
        let cheated_data_ = cheatnet_state.create_cheated_data(contract_address);
        cheatnet_state.update_cheats(&contract_address);
        cheated_data_
    };

    // region: Modified blockifier code
    // We skip recursion depth validation here.
    cheatnet_state
        .trace_data
        .enter_nested_call(entry_point.clone(), cheated_data);

    if let Some(cheat_status) = get_mocked_function_cheat_status(entry_point, cheatnet_state) {
        if let CheatStatus::Cheated(ret_data, _) = (*cheat_status).clone() {
            cheat_status.decrement_cheat_span();
            let ret_data_f252: Vec<Felt> =
                ret_data.iter().map(|datum| Felt::from_(*datum)).collect();
            cheatnet_state.trace_data.exit_nested_call(
                ExecutionResources::default(),
                u64::default(),
                HashMap::default(),
                CallResult::Success {
                    ret_data: ret_data_f252,
                },
                &[],
                None,
            );
            let tracked_resource = *context
                .tracked_resource_stack
                .last()
                .expect("Unexpected empty tracked resource.");
            return Ok(mocked_call_info(
                entry_point.clone(),
                ret_data.clone(),
                tracked_resource,
            ));
        }
    }
    // endregion

    // Validate contract is deployed.
    let storage_address = entry_point.storage_address;
    let storage_class_hash = state.get_class_hash_at(entry_point.storage_address)?;
    if storage_class_hash == ClassHash::default() {
        return Err(
            PreExecutionError::UninitializedStorageAddress(entry_point.storage_address).into(),
        );
    }
    let maybe_replacement_class = cheatnet_state
        .replaced_bytecode_contracts
        .get(&storage_address)
        .copied();

    let class_hash = entry_point
        .class_hash
        .or(maybe_replacement_class)
        .unwrap_or(storage_class_hash); // If not given, take the storage contract class hash.
    let compiled_class = state.get_compiled_class(class_hash)?;
    let current_tracked_resource = compiled_class.get_current_tracked_resource(context);

    // region: Modified blockifier code
    cheatnet_state
        .trace_data
        .set_class_hash_for_current_call(class_hash);
    // endregion

    // Hack to prevent version 0 attack on argent accounts.
    if context.tx_context.tx_info.version() == TransactionVersion(Felt::from(0_u8))
        && class_hash
            == TryFromHexStr::try_from_hex_str(FAULTY_CLASS_HASH)
                .expect("A class hash must be a felt.")
    {
        return Err(PreExecutionError::FraudAttempt.into());
    }

    let entry_point = ExecutableCallEntryPoint {
        class_hash,
        code_address: entry_point.code_address,
        entry_point_type: entry_point.entry_point_type,
        entry_point_selector: entry_point.entry_point_selector,
        calldata: entry_point.calldata.clone(),
        storage_address: entry_point.storage_address,
        caller_address: entry_point.caller_address,
        call_type: entry_point.call_type,
        initial_gas: entry_point.initial_gas,
    };

    let contract_class = state.get_compiled_class(class_hash)?;

    context.revert_infos.0.push(EntryPointRevertInfo::new(
        entry_point.storage_address,
        class_hash,
        context.n_emitted_events,
        context.n_sent_messages_to_l1,
    ));

    context
        .tracked_resource_stack
        .push(current_tracked_resource);
    // Region: Modified blockifier code
    let result = match contract_class {
        RunnableCompiledClass::V0(compiled_class_v0) => execute_entry_point_call_cairo0(
            entry_point.clone(),
            compiled_class_v0,
            state,
            cheatnet_state,
            context,
        ),
        RunnableCompiledClass::V1(compiled_class_v1) => execute_entry_point_call_cairo1(
            entry_point.clone(),
            &compiled_class_v1,
            state,
            cheatnet_state,
            context,
        ),
    };
    context
        .tracked_resource_stack
        .pop()
        .expect("Unexpected empty tracked resource.");

    // region: Modified blockifier code
    match evaluate_execution_result(
        result,
        entry_point.clone(),
        current_tracked_resource,
        cheatnet_state,
        is_revertable,
    ) {
        Ok(CallInfoWithExecutionData {
            call_info,
            syscall_usage,
            vm_trace,
        }) => {
            remove_syscall_resources_and_exit_non_error_call(
                &call_info,
                &syscall_usage,
                context,
                cheatnet_state,
                vm_trace,
                current_tracked_resource,
            );
            Ok(call_info)
        }
        Err(EntryPointExecutionErrorWithTrace { source: err, trace }) => {
            exit_error_call(&err, cheatnet_state, &entry_point, trace);
            Err(err)
        }
    }
    // endregion
}

fn evaluate_execution_result(
    result: ContractClassEntryPointExecutionResult,
    call: ExecutableCallEntryPoint,
    current_tracked_resource: TrackedResource,
    cheatnet_state: &mut CheatnetState,
    is_revertable: bool,
) -> ContractClassEntryPointExecutionResult {
    match result {
        Ok(res) => {
            if res.call_info.execution.failed && !is_revertable {
                clear_handled_errors(&res.call_info, cheatnet_state);
                return Err(EntryPointExecutionErrorWithTrace {
                    source: EntryPointExecutionError::ExecutionFailed {
                        error_trace: extract_trailing_cairo1_revert_trace(
                            &res.call_info,
                            Cairo1RevertHeader::Execution,
                        ),
                    },
                    trace: res.vm_trace,
                });
            }
            Ok(res)
        }
        Err(err) => {
            handle_entry_point_execution_error(err, call, current_tracked_resource, is_revertable)
        }
    }
}

fn handle_entry_point_execution_error(
    err: EntryPointExecutionErrorWithTrace,
    call: ExecutableCallEntryPoint,
    current_tracked_resource: TrackedResource,
    is_revertable: bool,
) -> ContractClassEntryPointExecutionResult {
    if let EntryPointExecutionError::PreExecutionError(pre_err) = &err.source {
        match pre_err {
            PreExecutionError::EntryPointNotFound(_)
            | PreExecutionError::NoEntryPointOfTypeFound(_)
                if is_revertable =>
            {
                return Ok(call_info_from_pre_execution_error(
                    call,
                    current_tracked_resource,
                    ENTRYPOINT_NOT_FOUND_ERROR,
                ));
            }
            PreExecutionError::InsufficientEntryPointGas if is_revertable => {
                return Ok(call_info_from_pre_execution_error(
                    call,
                    current_tracked_resource,
                    OUT_OF_GAS_ERROR,
                ));
            }
            _ => {}
        }
    }
    Err(err)
}

fn call_info_from_pre_execution_error(
    call: ExecutableCallEntryPoint,
    current_tracked_resource: TrackedResource,
    error_code: &str,
) -> CallInfoWithExecutionData {
    CallInfoWithExecutionData {
        call_info: CallInfo {
            call: call.into(),
            execution: CallExecution {
                retdata: Retdata(vec![Felt::from_hex(error_code).unwrap()]),
                failed: true,
                gas_consumed: 0,
                ..Default::default()
            },
            tracked_resource: current_tracked_resource,
            ..Default::default()
        },
        syscall_usage: SyscallUsageMap::default(),
        vm_trace: None,
    }
}

fn remove_syscall_resources_and_exit_non_error_call(
    call_info: &CallInfo,
    syscall_usage: &SyscallUsageMap,
    context: &mut EntryPointExecutionContext,
    cheatnet_state: &mut CheatnetState,
    vm_trace: Option<Vec<RelocatedTraceEntry>>,
    current_tracked_resource: TrackedResource,
) {
    let versioned_constants = context.tx_context.block_context.versioned_constants();
    // We don't want the syscall resources to pollute the results
    let mut resources = call_info.resources.clone();
    let mut gas_consumed = call_info.execution.gas_consumed;

    match current_tracked_resource {
        TrackedResource::CairoSteps => {
            resources -= &versioned_constants.get_additional_os_syscall_resources(syscall_usage);
        }
        TrackedResource::SierraGas => {
            gas_consumed -= get_syscalls_gas_consumed(syscall_usage, versioned_constants);
        }
    }

    let nested_syscall_usage_sum =
        aggregate_nested_syscall_usage(&cheatnet_state.trace_data.current_call_stack.top());
    let syscall_usage = sum_syscall_usage(nested_syscall_usage_sum, syscall_usage);
    cheatnet_state.trace_data.exit_nested_call(
        resources,
        gas_consumed,
        syscall_usage,
        CallResult::from_non_error(call_info),
        &call_info.execution.l2_to_l1_messages,
        vm_trace,
    );
}

fn exit_error_call(
    error: &EntryPointExecutionError,
    cheatnet_state: &mut CheatnetState,
    entry_point: &ExecutableCallEntryPoint,
    vm_trace: Option<Vec<RelocatedTraceEntry>>,
) {
    let identifier = match entry_point.call_type {
        CallType::Call => AddressOrClassHash::ContractAddress(entry_point.storage_address),
        CallType::Delegate => AddressOrClassHash::ClassHash(entry_point.class_hash),
    };
    cheatnet_state.trace_data.exit_nested_call(
        ExecutionResources::default(),
        u64::default(),
        HashMap::default(),
        CallResult::from_err(error, &identifier),
        &[],
        vm_trace,
    );
}

// blockifier/src/execution/entry_point.rs:366 (execute_constructor_entry_point)
pub fn execute_constructor_entry_point(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: &ConstructorContext,
    calldata: Calldata,
    remaining_gas: u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Ensure the class is declared (by reading it).
    let contract_class = state.get_compiled_class(ctor_context.class_hash)?;
    let Some(constructor_selector) = contract_class.constructor_selector() else {
        // Contract has no constructor.
        cheatnet_state
            .trace_data
            .add_deploy_without_constructor_node();
        return handle_empty_constructor(
            contract_class,
            context,
            ctor_context,
            calldata,
            remaining_gas,
        );
    };

    let mut constructor_call = CallEntryPoint {
        class_hash: None,
        code_address: ctor_context.code_address,
        entry_point_type: EntryPointType::Constructor,
        entry_point_selector: constructor_selector,
        calldata,
        storage_address: ctor_context.storage_address,
        caller_address: ctor_context.caller_address,
        call_type: CallType::Call,
        initial_gas: remaining_gas,
    };
    // region: Modified blockifier code
    execute_call_entry_point(&mut constructor_call, state, cheatnet_state, context, false)
    // endregion
}

fn get_mocked_function_cheat_status<'a>(
    call: &CallEntryPoint,
    cheatnet_state: &'a mut CheatnetState,
) -> Option<&'a mut CheatStatus<Vec<Felt>>> {
    if call.call_type == CallType::Delegate {
        return None;
    }

    cheatnet_state
        .mocked_functions
        .get_mut(&call.storage_address)
        .and_then(|contract_functions| contract_functions.get_mut(&call.entry_point_selector))
}

fn mocked_call_info(
    call: CallEntryPoint,
    ret_data: Vec<Felt>,
    tracked_resource: TrackedResource,
) -> CallInfo {
    CallInfo {
        call: CallEntryPoint {
            class_hash: Some(call.class_hash.unwrap_or_default()),
            ..call
        },
        execution: CallExecution {
            retdata: Retdata(ret_data),
            events: vec![],
            l2_to_l1_messages: vec![],
            cairo_native: false,
            failed: false,
            gas_consumed: 0,
        },
        resources: ExecutionResources::default(),
        tracked_resource,
        inner_calls: vec![],
        storage_access_tracker: StorageAccessTracker::default(),
        accessed_contract_addresses: HashSet::default(),
        builtin_counters: HashMap::default(),
    }
}

fn aggregate_nested_syscall_usage(trace: &Rc<RefCell<CallTrace>>) -> SyscallUsageMap {
    let mut result = SyscallUsageMap::new();
    for nested_call_node in &trace.borrow().nested_calls {
        if let CallTraceNode::EntryPointCall(nested_call) = nested_call_node {
            let sub_trace_counter = aggregate_syscall_usage(nested_call);
            result = sum_syscall_usage(result, &sub_trace_counter);
        }
    }
    result
}

fn aggregate_syscall_usage(trace: &Rc<RefCell<CallTrace>>) -> SyscallUsageMap {
    let mut result = trace.borrow().used_syscalls.clone();
    for nested_call_node in &trace.borrow().nested_calls {
        if let CallTraceNode::EntryPointCall(nested_call) = nested_call_node {
            let sub_trace_counter = aggregate_nested_syscall_usage(nested_call);
            result = sum_syscall_usage(result, &sub_trace_counter);
        }
    }
    result
}

#[derive(Debug, Error)]
#[error("{}", source)]
pub struct EntryPointExecutionErrorWithTrace {
    pub source: EntryPointExecutionError,
    pub trace: Option<Vec<RelocatedTraceEntry>>,
}

impl<T> From<T> for EntryPointExecutionErrorWithTrace
where
    T: Into<EntryPointExecutionError>,
{
    fn from(value: T) -> Self {
        Self {
            source: value.into(),
            trace: None,
        }
    }
}

pub(crate) fn extract_trace_and_register_errors(
    source: EntryPointExecutionError,
    class_hash: ClassHash,
    runner: &mut CairoRunner,
    cheatnet_state: &mut CheatnetState,
) -> EntryPointExecutionErrorWithTrace {
    let trace = get_relocated_vm_trace(runner);
    let pcs = runner.vm.get_reversed_pc_traceback();
    cheatnet_state.register_error(class_hash, pcs);

    EntryPointExecutionErrorWithTrace {
        source,
        trace: Some(trace),
    }
}

/// This helper function is used for backtrace to avoid displaying errors that were already handled
/// It clears the errors for all contracts that failed with a different panic data than the root call
/// Note: This may not be accurate if a panic was initially handled and then the function panicked
/// again with the identical panic data
fn clear_handled_errors(root_call: &CallInfo, cheatnet_state: &mut CheatnetState) {
    let contracts_matching_root_error = get_contracts_with_matching_error(root_call);

    cheatnet_state
        .encountered_errors
        .clone()
        .keys()
        .for_each(|&class_hash| {
            if !contracts_matching_root_error.contains(&class_hash) {
                cheatnet_state.clear_error(class_hash);
            }
        });
}

/// Collects all contracts that have matching error with the root call
fn get_contracts_with_matching_error(root_call: &CallInfo) -> HashSet<ClassHash> {
    let mut contracts_matching_root_error = HashSet::new();
    let mut failed_matching_calls: Vec<&CallInfo> = vec![root_call];

    while let Some(call_info) = failed_matching_calls.pop() {
        if let Some(class_hash) = call_info.call.class_hash {
            contracts_matching_root_error.insert(class_hash);
            failed_matching_calls.extend(get_inner_calls_with_matching_panic_data(
                call_info,
                &root_call.execution.retdata.0,
            ));
        }
    }

    contracts_matching_root_error
}

fn get_inner_calls_with_matching_panic_data<'a>(
    call_info: &'a CallInfo,
    root_retdata: &[Felt],
) -> Vec<&'a CallInfo> {
    call_info
        .inner_calls
        .iter()
        .filter(|call| call.execution.failed && root_retdata.starts_with(&call.execution.retdata.0))
        .collect()
}
