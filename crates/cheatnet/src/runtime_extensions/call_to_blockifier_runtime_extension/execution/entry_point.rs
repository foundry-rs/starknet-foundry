use super::cairo1_execution::execute_entry_point_call_cairo1;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::deprecated::cairo0_execution::execute_entry_point_call_cairo0;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::execution_utils::{exit_error_call, resolve_cheated_data_for_call, update_trace_data};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallSuccess;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::common::get_relocated_vm_trace;
#[cfg(feature = "cairo-native")]
use crate::runtime_extensions::native::execution::execute_entry_point_call_native;
use crate::state::CheatStatus;
use blockifier::execution::call_info::{BuiltinCounterMap, CallExecution, Retdata, StorageAccessTracker};
use blockifier::execution::contract_class::{RunnableCompiledClass, TrackedResource};
use blockifier::execution::entry_point::EntryPointRevertInfo;
use blockifier::execution::execution_utils::update_remaining_gas;
use blockifier::execution::stack_trace::{
    extract_trailing_cairo1_revert_trace, Cairo1RevertHeader,
};
use blockifier::execution::syscalls::hint_processor::{
    ENTRYPOINT_NOT_FOUND_ERROR, OUT_OF_GAS_ERROR,
};
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use blockifier::{
    execution::{
        call_info::CallInfo,
        entry_point::{
            handle_empty_constructor, CallEntryPoint, CallType, ConstructorContext,
            EntryPointExecutionContext, EntryPointExecutionResult, FAULTY_CLASS_HASH,
        },
        errors::{EntryPointExecutionError, PreExecutionError},
    },
    state::state_api::State,
};
use cairo_vm::vm::runners::cairo_runner::{CairoRunner, ExecutionResources};
use conversions::FromConv;
use shared::vm::VirtualMachineExt;
use starknet_api::execution_resources::GasAmount;
use starknet_api::{
    contract_class::EntryPointType,
    core::ClassHash,
    transaction::{fields::Calldata, TransactionVersion},
};
use starknet_types_core::felt::Felt;
use std::collections::HashSet;

pub(crate) type ContractClassEntryPointExecutionResult =
    Result<CallInfoWithExecutionData, EntryPointExecutionError>;

pub(crate) struct CallInfoWithExecutionData {
    pub call_info: CallInfo,
    pub syscall_usage_vm_resources: SyscallUsageMap,
    pub syscall_usage_sierra_gas: SyscallUsageMap,
}

#[derive(Default)]
pub struct ExecuteCallEntryPointExtraOptions {
    pub trace_data_handled_by_revert_call: bool,
}

// blockifier/src/execution/entry_point (CallEntryPoint::execute)
#[expect(clippy::too_many_lines)]
pub fn execute_call_entry_point(
    entry_point: &mut CallEntryPoint, // Instead of 'self'
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    remaining_gas: &mut u64,
    opts: &ExecuteCallEntryPointExtraOptions,
) -> EntryPointExecutionResult<CallInfo> {
    // region: Modified blockifier code
    // We skip recursion depth validation here.
    if !opts.trace_data_handled_by_revert_call {
        let cheated_data = resolve_cheated_data_for_call(entry_point, cheatnet_state);
        cheatnet_state
            .trace_data
            .enter_nested_call(entry_point.clone(), cheated_data.clone());
    }

    if let Some(cheat_status) = get_mocked_function_cheat_status(entry_point, cheatnet_state)
        && let CheatStatus::Cheated(ret_data, _) = (*cheat_status).clone()
    {
        cheat_status.decrement_cheat_span();
        let ret_data_f252: Vec<Felt> = ret_data.iter().map(|datum| Felt::from_(*datum)).collect();
        cheatnet_state.trace_data.update_current_call(
            ExecutionResources::default(),
            u64::default(),
            SyscallUsageMap::default(),
            SyscallUsageMap::default(),
            Ok(CallSuccess {
                ret_data: ret_data_f252,
            }),
            &[],
            vec![],
            vec![],
        );

        if !opts.trace_data_handled_by_revert_call {
            cheatnet_state.trace_data.exit_nested_call();
        }

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
    // endregion

    // Validate contract is deployed.
    let storage_class_hash = state.get_class_hash_at(entry_point.storage_address)?;
    if storage_class_hash == ClassHash::default() {
        return Err(
            PreExecutionError::UninitializedStorageAddress(entry_point.storage_address).into(),
        );
    }

    // region: Modified blockifier code
    let maybe_replacement_class = cheatnet_state
        .replaced_bytecode_contracts
        .get(&entry_point.storage_address)
        .copied();
    let class_hash = entry_point
        .class_hash
        .or(maybe_replacement_class)
        .unwrap_or(storage_class_hash); // If not given, take the storage contract class hash.
    // endregion

    let compiled_class = state.get_compiled_class(class_hash)?;
    let current_tracked_resource = compiled_class.get_current_tracked_resource(context);

    // region: Modified blockifier code
    cheatnet_state
        .trace_data
        .set_class_hash_for_current_call(class_hash);
    // endregion

    // Hack to prevent version 0 attack on ready (formerly argent) accounts.
    if context.tx_context.tx_info.version() == TransactionVersion::ZERO
        && class_hash
            == ClassHash(Felt::from_hex(FAULTY_CLASS_HASH).expect("A class hash must be a felt."))
    {
        return Err(PreExecutionError::FraudAttempt.into());
    }

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
    let entry_point = entry_point.clone().into_executable(class_hash);
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
        #[cfg(feature = "cairo-native")]
        RunnableCompiledClass::V1Native(native_compiled_class_v1) => {
            if context.tracked_resource_stack.last() == Some(&TrackedResource::CairoSteps) {
                execute_entry_point_call_cairo1(
                    entry_point.clone(),
                    &native_compiled_class_v1.casm(),
                    state,
                    cheatnet_state,
                    context,
                )
            } else {
                execute_entry_point_call_native(
                    &entry_point,
                    &native_compiled_class_v1,
                    state,
                    cheatnet_state,
                    context,
                )
            }
        }
    };
    context
        .tracked_resource_stack
        .pop()
        .expect("Unexpected empty tracked resource.");

    match result {
        Ok(res) => {
            if res.call_info.execution.failed && !context.versioned_constants().enable_reverts {
                let err = EntryPointExecutionError::ExecutionFailed {
                    error_trace: extract_trailing_cairo1_revert_trace(
                        &res.call_info,
                        Cairo1RevertHeader::Execution,
                    ),
                };
                exit_error_call(&err, cheatnet_state, &entry_point);
                return Err(err);
            }
            update_remaining_gas(remaining_gas, &res.call_info);
            update_trace_data(
                &res.call_info,
                &res.syscall_usage_vm_resources,
                &res.syscall_usage_sierra_gas,
                cheatnet_state,
            );

            if !opts.trace_data_handled_by_revert_call {
                cheatnet_state.trace_data.exit_nested_call();
            }

            Ok(res.call_info)
        }
        Err(EntryPointExecutionError::PreExecutionError(err))
            if context.versioned_constants().enable_reverts =>
        {
            let error_code = match err {
                PreExecutionError::EntryPointNotFound(_)
                | PreExecutionError::NoEntryPointOfTypeFound(_) => ENTRYPOINT_NOT_FOUND_ERROR,
                PreExecutionError::InsufficientEntryPointGas => OUT_OF_GAS_ERROR,
                _ => return Err(err.into()),
            };
            Ok(CallInfo {
                call: entry_point.into(),
                execution: CallExecution {
                    retdata: Retdata(vec![Felt::from_hex(error_code).unwrap()]),
                    failed: true,
                    gas_consumed: 0,
                    ..CallExecution::default()
                },
                tracked_resource: current_tracked_resource,
                ..CallInfo::default()
            })
        }
        Err(err) => {
            exit_error_call(&err, cheatnet_state, &entry_point);
            Err(err)
        }
    }
    // endregion
}

// blockifier/src/execution/entry_point (CallEntryPoint::non_reverting_execute)
pub fn non_reverting_execute_call_entry_point(
    entry_point: &mut CallEntryPoint, // Instead of 'self'
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    remaining_gas: &mut u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Region: Modified blockifier code
    let cheated_data = resolve_cheated_data_for_call(entry_point, cheatnet_state);
    cheatnet_state
        .trace_data
        .enter_nested_call(entry_point.clone(), cheated_data.clone());
    // endregion

    let execution_result = execute_call_entry_point(
        entry_point,
        state,
        cheatnet_state,
        context,
        remaining_gas,
        &ExecuteCallEntryPointExtraOptions {
            trace_data_handled_by_revert_call: true,
        },
    );

    if let Ok(call_info) = &execution_result {
        // Update revert gas tracking (for completeness - value will not be used unless the tx
        // is reverted).
        context
            .sierra_gas_revert_tracker
            .update_with_next_remaining_gas(call_info.tracked_resource, GasAmount(*remaining_gas));
        // If the execution of the outer call failed, revert the transaction.
        if call_info.execution.failed {
            // Region: Modified blockifier code
            clear_handled_errors(call_info, cheatnet_state);
            let err = EntryPointExecutionError::ExecutionFailed {
                error_trace: extract_trailing_cairo1_revert_trace(
                    call_info,
                    Cairo1RevertHeader::Execution,
                ),
            };
            // Note: Class hash in the entry point below does not matter, as `exit_error_call` does not update it in the trace.
            exit_error_call(
                &err,
                cheatnet_state,
                &entry_point
                    .clone()
                    .into_executable(entry_point.class_hash.unwrap_or_default()),
            );
            return Err(err);
        }
        cheatnet_state.trace_data.exit_nested_call();
        // endregion
    }

    execution_result
}

// blockifier/src/execution/entry_point.rs (execute_constructor_entry_point)
pub fn execute_constructor_entry_point(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: &ConstructorContext,
    calldata: Calldata,
    remaining_gas: &mut u64,
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
            *remaining_gas,
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
        initial_gas: *remaining_gas,
    };
    // region: Modified blockifier code
    non_reverting_execute_call_entry_point(
        &mut constructor_call,
        state,
        cheatnet_state,
        context,
        remaining_gas,
    )
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
        builtin_counters: BuiltinCounterMap::default(),
        syscalls_usage: SyscallUsageMap::default(),
    }
}

pub(crate) fn extract_trace_and_register_errors(
    class_hash: ClassHash,
    runner: &mut CairoRunner,
    cheatnet_state: &mut CheatnetState,
) {
    let trace = get_relocated_vm_trace(runner);
    cheatnet_state
        .trace_data
        .set_vm_trace_for_current_call(trace);

    let pcs = runner.vm.get_reversed_pc_traceback();
    cheatnet_state.register_error(class_hash, pcs);
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
