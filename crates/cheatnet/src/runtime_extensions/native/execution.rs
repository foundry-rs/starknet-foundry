use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    CallInfoWithExecutionData, ContractClassEntryPointExecutionResult,
};
use crate::runtime_extensions::native::native_syscall_handler::CheatableNativeSyscallHandler;
use crate::state::CheatnetState;
use blockifier::execution::call_info::{
    CairoPrimitiveCounterMap, CallExecution, CallInfo, OpcodeName, Retdata,
    cairo_primitive_counter_map,
};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::entry_point::{
    EntryPointExecutionContext, EntryPointExecutionResult, ExecutableCallEntryPoint,
};
use blockifier::execution::errors::{
    EntryPointExecutionError, PostExecutionError, PreExecutionError,
};
use blockifier::execution::native::contract_class::NativeCompiledClassV1;
use blockifier::execution::native::syscall_handler::NativeSyscallHandler;
use blockifier::state::state_api::State;
use blockifier::transaction::objects::ExecutionResourcesTraits;
use blockifier::utils::add_maps;
use cairo_native::execution_result::{BuiltinStats, ContractExecutionResult};
use cairo_native::utils::BuiltinCosts;
use cairo_vm::types::builtin_name::BuiltinName;
use std::collections::HashMap;
use std::default::Default;

pub(crate) fn execute_entry_point_call_native(
    call: &ExecutableCallEntryPoint,
    native_compiled_class_v1: &NativeCompiledClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let mut syscall_handler = CheatableNativeSyscallHandler {
        cheatnet_state,
        native_syscall_handler: &mut NativeSyscallHandler::new(call.clone(), state, context),
    };

    let call_info = execute_entry_point_call(call, native_compiled_class_v1, &mut syscall_handler)?;

    let syscall_usage = &syscall_handler.native_syscall_handler.base.syscalls_usage;

    Ok(CallInfoWithExecutionData {
        call_info,
        // Native execution doesn't support VM resources.
        // If we got to this point, it means tracked resources are SierraGas.
        syscall_usage_vm_resources: HashMap::default(),
        syscall_usage_sierra_gas: syscall_usage.clone(),
    })
}

// Based on https://github.com/software-mansion-labs/sequencer/blob/b6d1c0b354d84225ab9c47f8ff28663d22e84d19/crates/blockifier/src/execution/native/entry_point_execution.rs#L20
fn execute_entry_point_call(
    call: &ExecutableCallEntryPoint,
    compiled_class: &NativeCompiledClassV1,
    // region: Modified blockifier code
    syscall_handler: &mut CheatableNativeSyscallHandler,
    // endregion
) -> EntryPointExecutionResult<CallInfo> {
    let entry_point = compiled_class.get_entry_point(&call.type_and_selector())?;

    let gas_costs = &syscall_handler
        .native_syscall_handler
        .base
        .context
        .gas_costs();
    let builtin_costs = BuiltinCosts {
        r#const: 1,
        pedersen: gas_costs.builtins.pedersen,
        bitwise: gas_costs.builtins.bitwise,
        ecop: gas_costs.builtins.ecop,
        poseidon: gas_costs.builtins.poseidon,
        add_mod: gas_costs.builtins.add_mod,
        mul_mod: gas_costs.builtins.mul_mod,
        blake: gas_costs.builtins.blake,
    };

    // Pre-charge entry point's initial budget to ensure sufficient gas for executing a minimal
    // entry point code. When redepositing is used, the entry point is aware of this pre-charge
    // and adjusts the gas counter accordingly if a smaller amount of gas is required.
    let initial_budget = syscall_handler
        .native_syscall_handler
        .base
        .context
        .gas_costs()
        .base
        .entry_point_initial_budget;
    let call_initial_gas = syscall_handler
        .native_syscall_handler
        .base
        .call
        .initial_gas
        .checked_sub(initial_budget)
        .ok_or(PreExecutionError::InsufficientEntryPointGas)?;

    let execution_result = compiled_class.executor.run(
        entry_point.selector.0,
        &syscall_handler
            .native_syscall_handler
            .base
            .call
            .calldata
            .0
            .clone(),
        call_initial_gas,
        Some(builtin_costs),
        &mut *syscall_handler,
    );

    syscall_handler.native_syscall_handler.finalize();

    let call_result = execution_result.map_err(EntryPointExecutionError::NativeUnexpectedError)?;

    // TODO(#3790) consider modifying this so it doesn't use take internally
    if let Some(error) = syscall_handler.unrecoverable_error() {
        return Err(EntryPointExecutionError::NativeUnrecoverableError(
            Box::new(error),
        ));
    }

    create_callinfo(call_result, syscall_handler)
}

// Copied from https://github.com/software-mansion-labs/sequencer/blob/b6d1c0b354d84225ab9c47f8ff28663d22e84d19/crates/blockifier/src/execution/native/entry_point_execution.rs#L73
fn create_callinfo(
    call_result: ContractExecutionResult,
    syscall_handler: &mut CheatableNativeSyscallHandler<'_>,
) -> Result<CallInfo, EntryPointExecutionError> {
    let remaining_gas = call_result.remaining_gas;

    if remaining_gas > syscall_handler.native_syscall_handler.base.call.initial_gas {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: format!(
                "Unexpected remaining gas. Used gas is greater than initial gas: {} > {}",
                remaining_gas, syscall_handler.native_syscall_handler.base.call.initial_gas
            ),
        }
        .into());
    }

    let gas_consumed = syscall_handler.native_syscall_handler.base.call.initial_gas - remaining_gas;
    let vm_resources = CallInfo::summarize_vm_resources(
        syscall_handler
            .native_syscall_handler
            .base
            .inner_calls
            .iter(),
    );

    // Retrieve the builtin counts from the syscall handler
    let version_constants = syscall_handler
        .native_syscall_handler
        .base
        .context
        .versioned_constants();
    let syscall_builtins = version_constants
        .get_additional_os_syscall_resources(
            &syscall_handler.native_syscall_handler.base.syscalls_usage,
        )
        .filter_unused_builtins()
        .prover_builtins();
    let mut entry_point_primitive_counters =
        builtin_stats_to_primitive_counters(call_result.builtin_stats);
    add_maps(
        &mut entry_point_primitive_counters,
        &cairo_primitive_counter_map(syscall_builtins),
    );

    Ok(CallInfo {
        call: syscall_handler
            .native_syscall_handler
            .base
            .call
            .clone()
            .into(),
        execution: CallExecution {
            retdata: Retdata(call_result.return_values),
            events: syscall_handler.native_syscall_handler.base.events.clone(),
            cairo_native: true,
            l2_to_l1_messages: syscall_handler
                .native_syscall_handler
                .base
                .l2_to_l1_messages
                .clone(),
            failed: call_result.failure_flag,
            gas_consumed,
        },
        resources: vm_resources,
        inner_calls: syscall_handler
            .native_syscall_handler
            .base
            .inner_calls
            .clone(),
        storage_access_tracker: syscall_handler
            .native_syscall_handler
            .base
            .storage_access_tracker
            .clone(),
        tracked_resource: TrackedResource::SierraGas,
        builtin_counters: entry_point_primitive_counters,
        syscalls_usage: syscall_handler
            .native_syscall_handler
            .base
            .syscalls_usage
            .clone(),
    })
}

// Copied from https://github.com/starkware-libs/sequencer/blob/blockifier-v0.18.0-rc.1/crates/blockifier/src/execution/native/entry_point_execution.rs#L130
fn builtin_stats_to_primitive_counters(stats: BuiltinStats) -> CairoPrimitiveCounterMap {
    let builtins = [
        (BuiltinName::range_check, stats.range_check),
        (BuiltinName::pedersen, stats.pedersen),
        (BuiltinName::bitwise, stats.bitwise),
        (BuiltinName::ec_op, stats.ec_op),
        (BuiltinName::poseidon, stats.poseidon),
        (BuiltinName::range_check96, stats.range_check96),
        (BuiltinName::add_mod, stats.add_mod),
        (BuiltinName::mul_mod, stats.mul_mod),
    ];
    let opcodes = [(OpcodeName::blake, stats.blake)];

    builtins
        .into_iter()
        .map(|(builtin_name, count)| (builtin_name.into(), count))
        .chain(
            opcodes
                .into_iter()
                .map(|(opcode_name, count): (OpcodeName, _)| (opcode_name.into(), count)),
        )
        .filter(|(_, count)| *count > 0)
        .collect()
}
