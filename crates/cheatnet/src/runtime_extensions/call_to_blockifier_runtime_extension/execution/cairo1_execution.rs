use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    CallInfoWithExecutionData, ContractClassEntryPointExecutionResult,
    extract_trace_and_register_errors,
};
use crate::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use crate::runtime_extensions::common::get_relocated_vm_trace;
use blockifier::execution::call_info::{CallExecution, CallInfo};
use blockifier::execution::contract_class::{CompiledClassV1, TrackedResource};
use blockifier::execution::entry_point::ExecutableCallEntryPoint;
use blockifier::execution::entry_point_execution::{
    CallResult, ExecutionRunnerMode, VmExecutionContext, extract_vm_resources, finalize_runner,
    initialize_execution_context_with_runner_mode, prepare_call_arguments, total_vm_resources,
};
use blockifier::execution::errors::PostExecutionError;
use blockifier::execution::execution_utils::read_execution_retdata;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use blockifier::transaction::objects::ExecutionResourcesTraits;
use blockifier::{
    execution::{
        contract_class::EntryPointV1, entry_point::EntryPointExecutionContext,
        errors::EntryPointExecutionError, execution_utils::Args,
    },
    state::state_api::State,
};
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::{
    hint_processor::hint_processor_definition::HintProcessor,
    vm::runners::cairo_runner::{CairoArg, CairoRunner},
};
use num_traits::ToPrimitive;
use runtime::{ExtendedRuntime, StarknetRuntime};

pub fn finalize_execution(
    mut runner: CairoRunner,
    mut syscall_handler: SyscallHintProcessor<'_>,
    n_total_args: usize,
    program_extra_data_length: usize,
    tracked_resource: TrackedResource,
) -> Result<CallInfo, PostExecutionError> {
    finalize_runner(&mut runner, n_total_args, program_extra_data_length)?;
    syscall_handler
        .read_only_segments
        .mark_as_accessed(&mut runner)?;

    let call_result = get_call_result(
        &runner,
        &syscall_handler,
        &tracked_resource,
    )?;

    // Take into account the resources of the current call, without inner calls.
    // Has to happen after marking holes in segments as accessed.
    let vm_resources_without_inner_calls = extract_vm_resources(&runner, &syscall_handler)?;

    let tracked_vm_resources_without_inner_calls = match tracked_resource {
        TrackedResource::CairoSteps => &vm_resources_without_inner_calls,
        TrackedResource::SierraGas => &ExecutionResources::default(),
    };

    syscall_handler.finalize();

    let vm_resources = total_vm_resources(
        tracked_vm_resources_without_inner_calls,
        &syscall_handler.base.inner_calls,
    );

    let syscall_handler_base = syscall_handler.base;

    Ok(CallInfo {
        call: syscall_handler_base.call.into(),
        execution: CallExecution {
            retdata: call_result.retdata,
            events: syscall_handler_base.events,
            l2_to_l1_messages: syscall_handler_base.l2_to_l1_messages,
            cairo_native: false,
            failed: call_result.failed,
            gas_consumed: call_result.gas_consumed,
        },
        inner_calls: syscall_handler_base.inner_calls,
        tracked_resource,
        resources: vm_resources,
        storage_access_tracker: syscall_handler_base.storage_access_tracker,
        builtin_counters: vm_resources_without_inner_calls.prover_builtins(),
    })
}

pub fn get_call_result(
    runner: &CairoRunner,
    syscall_handler: &SyscallHintProcessor<'_>,
    tracked_resource: &TrackedResource,
) -> Result<CallResult, PostExecutionError> {
    let return_result = runner.vm.get_return_values(5)?;
    // Corresponds to the Cairo 1.0 enum:
    // enum PanicResult<Array::<felt>> { Ok: Array::<felt>, Err: Array::<felt>, }.
    let [failure_flag, retdata_start, retdata_end]: &[MaybeRelocatable; 3] = (&return_result[2..])
        .try_into()
        .expect("Return values must be of size 3.");

    let failed = if *failure_flag == MaybeRelocatable::from(0) {
        false
    } else if *failure_flag == MaybeRelocatable::from(1) {
        true
    } else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Failure flag expected to be either 0 or 1.".to_string(),
        });
    };

    let retdata_size = retdata_end.sub(retdata_start)?;
    // TODO(spapini): Validate implicits.

    let gas = &return_result[0];
    let MaybeRelocatable::Int(gas) = gas else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Error extracting return data.".to_string(),
        });
    };
    let gas = gas
        .to_u64()
        .ok_or(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        })?;

    if gas > syscall_handler.base.call.initial_gas {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        });
    }

    let gas_consumed = match tracked_resource {
        // Do not count Sierra gas in CairoSteps mode.
        TrackedResource::CairoSteps => 0,
        TrackedResource::SierraGas => syscall_handler.base.call.initial_gas - gas,
    };
    println!(
        "initial gas: {}, remianing gas: {}, gas consumed: {}",
        syscall_handler.base.call.initial_gas, gas, gas_consumed
    );
    Ok(CallResult {
        failed,
        retdata: read_execution_retdata(runner, retdata_size, retdata_start)?,
        gas_consumed,
    })
}

// blockifier/src/execution/cairo1_execution.rs:48 (execute_entry_point_call)
#[expect(clippy::result_large_err)]
pub(crate) fn execute_entry_point_call_cairo1(
    call: ExecutableCallEntryPoint,
    compiled_class_v1: &CompiledClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    let tracked_resource = *context
        .tracked_resource_stack
        .last()
        .expect("Unexpected empty tracked resource.");
    let entry_point_initial_budget = context.gas_costs().base.entry_point_initial_budget;

    let class_hash = call.class_hash;

    let VmExecutionContext {
        mut runner,
        mut syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context_with_runner_mode(
        call,
        compiled_class_v1,
        state,
        context,
        ExecutionRunnerMode::Tracing,
    )?;

    let args = prepare_call_arguments(
        &syscall_handler.base.call,
        &mut runner,
        initial_syscall_ptr,
        &mut syscall_handler.read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;
    let n_total_args = args.len();

    // region: Modified blockifier code

    let mut cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension { cheatnet_state },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            panic_traceback: None,
        },
    };

    // Execute.
    cheatable_run_entry_point(
        &mut runner,
        &mut cheatable_runtime,
        &entry_point,
        &args,
        program_extra_data_length,
    )
    .map_err(|source| {
        extract_trace_and_register_errors(
            source,
            class_hash,
            &mut runner,
            cheatable_runtime.extension.cheatnet_state,
        )
    })?;

    let trace = get_relocated_vm_trace(&mut runner);

    // Syscall usage here is flat, meaning it only includes syscalls from current call
    let syscall_usage = cheatable_runtime
        .extended_runtime
        .hint_handler
        .base
        .syscalls_usage
        .clone();

    let call_info = finalize_execution(
        runner,
        cheatable_runtime.extended_runtime.hint_handler,
        n_total_args,
        program_extra_data_length,
        tracked_resource,
    )?;

    if call_info.execution.failed {
        // fallback to the last pc in the trace if user did not set `panic-backtrace = true` in `Scarb.toml`
        let pcs = if let Some(panic_traceback) = cheatable_runtime.extended_runtime.panic_traceback
        {
            panic_traceback
        } else {
            trace
                .last()
                .map(|last| vec![last.pc])
                .expect("trace should have at least one entry")
        };
        cheatable_runtime
            .extension
            .cheatnet_state
            .register_error(class_hash, pcs);
    }

    let (syscall_usage_vm_resources, syscall_usage_sierra_gas) = match tracked_resource {
        TrackedResource::CairoSteps => (syscall_usage, SyscallUsageMap::default()),
        TrackedResource::SierraGas => (SyscallUsageMap::default(), syscall_usage),
    };

    println!(
        "GAS CONSUMED {:?} SYSCALL USAGE: {:?}",
        call_info.execution.gas_consumed, syscall_usage_sierra_gas
    );

    Ok(CallInfoWithExecutionData {
        call_info,
        syscall_usage_vm_resources,
        syscall_usage_sierra_gas,
        vm_trace: Some(trace),
    })
    // endregion
}

// crates/blockifier/src/execution/cairo1_execution.rs:236 (run_entry_point)
#[expect(clippy::result_large_err)]
pub fn cheatable_run_entry_point(
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: &EntryPointV1,
    args: &Args,
    program_segment_size: usize,
) -> Result<(), EntryPointExecutionError> {
    // region: Modified blockifier code
    // Opposite to blockifier
    let verify_secure = false;
    // endregion
    let args: Vec<&CairoArg> = args.iter().collect();

    runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        verify_secure,
        Some(program_segment_size),
        hint_processor,
    )?;

    // region: Modified blockifier code
    // Relocate trace to then collect it
    runner.relocate(true).map_err(CairoRunError::from)?;
    // endregion

    Ok(())
}
