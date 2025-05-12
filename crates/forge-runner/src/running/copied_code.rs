// TODO(#3293) Remove this file once the copied code is upstreamed to blockifier.
//! Module containing copied code to be upstreamed to blockifier

use blockifier::execution::call_info::CallInfo;
use blockifier::execution::contract_class::{EntryPointV1, TrackedResource};
use blockifier::execution::entry_point::EntryPointExecutionResult;
use blockifier::execution::entry_point_execution::CallResult;
use blockifier::execution::errors::{
    EntryPointExecutionError, PostExecutionError, PreExecutionError,
};
use blockifier::execution::execution_utils::{
    Args, ReadOnlySegments, SEGMENT_ARENA_BUILTIN_SIZE, read_execution_retdata, write_felt,
    write_maybe_relocatable,
};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::versioned_constants::GasCosts;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::builtin_runner::BuiltinRunner;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner, ExecutionResources};
use cairo_vm::vm::security::verify_secure_runner;
use num_traits::{ToPrimitive, Zero};
use starknet_types_core::felt::Felt;

// TODO to be upstreamed to blockifer
#[allow(clippy::needless_pass_by_value)]
// Reason copied: Signature change
/// Runs the runner from the given PC.
pub(crate) fn run_entry_point<HP: HintProcessor>(
    runner: &mut CairoRunner,
    // Modified code
    // hint_processor: &mut SyscallHintProcessor<'_>,
    hint_processor: &mut HP,
    entry_point: EntryPointV1,
    args: Args,
    program_segment_size: usize,
) -> EntryPointExecutionResult<()> {
    // Note that we run `verify_secure_runner` manually after filling the holes in the rc96 segment.
    let verify_secure = false;
    let args: Vec<&CairoArg> = args.iter().collect();
    runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        verify_secure,
        Some(program_segment_size),
        hint_processor,
    )?;

    maybe_fill_holes(entry_point, runner)?;

    verify_secure_runner(runner, false, Some(program_segment_size))
        .map_err(CairoRunError::VirtualMachine)?;

    Ok(())
}

// TODO to be upstreamed to blockifer
#[allow(clippy::items_after_statements, clippy::needless_pass_by_value)]
// Reason copied: Required by `run_entry_point`
/// Fills the holes after running the entry point.
/// Currently only fills the holes in the rc96 segment.
fn maybe_fill_holes(
    entry_point: EntryPointV1,
    runner: &mut CairoRunner,
) -> Result<(), EntryPointExecutionError> {
    let Some(rc96_offset) = entry_point
        .builtins
        .iter()
        .rev()
        .position(|name| *name == BuiltinName::range_check96)
    else {
        return Ok(());
    };
    let rc96_builtin_runner = runner
        .vm
        .get_builtin_runners()
        .iter()
        .find_map(|builtin| {
            if let BuiltinRunner::RangeCheck96(rc96_builtin_runner) = builtin {
                Some(rc96_builtin_runner)
            } else {
                None
            }
        })
        .expect("RangeCheck96 builtin runner not found.");

    // 'EntryPointReturnValues' is returned after the implicits and its size is 5,
    // So the last implicit is at offset 5 + 1.
    const IMPLICITS_OFFSET: usize = 6;
    let rc_96_stop_ptr = (runner.vm.get_ap() - (IMPLICITS_OFFSET + rc96_offset))
        .map_err(|err| CairoRunError::VirtualMachine(VirtualMachineError::Math(err)))?;

    let rc96_base = rc96_builtin_runner.base();
    let rc96_segment: isize = rc96_base
        .try_into()
        .expect("Builtin segment index must fit in isize.");

    let Relocatable {
        segment_index: rc96_stop_segment,
        offset: stop_offset,
    } = runner
        .vm
        .get_relocatable(rc_96_stop_ptr)
        .map_err(CairoRunError::MemoryError)?;
    assert_eq!(rc96_stop_segment, rc96_segment);

    // Update `segment_used_sizes` to include the holes.
    runner
        .vm
        .segments
        .segment_used_sizes
        .as_mut()
        .expect("Segments used sizes should be calculated at this point")[rc96_base] = stop_offset;

    for offset in 0..stop_offset {
        match runner.vm.insert_value(
            Relocatable {
                segment_index: rc96_segment,
                offset,
            },
            Felt::zero(),
        ) {
            // If the value is already set, ignore the error.
            Ok(()) | Err(MemoryError::InconsistentMemory(_)) => {}
            Err(err) => panic!("Unexpected error when filling holes: {err}."),
        }
    }

    Ok(())
}

// TODO to be upstreamed to blockifer
// Reason copied: Private function
pub(crate) fn prepare_program_extra_data(
    runner: &mut CairoRunner,
    // contract_class: &CompiledClassV1,
    bytecode_length: usize,
    read_only_segments: &mut ReadOnlySegments,
    gas_costs: &GasCosts,
) -> Result<usize, PreExecutionError> {
    // Create the builtin cost segment, the builtin order should be the same as the price builtin
    // array in the os in compiled_class.cairo in load_compiled_class_facts.
    let builtin_price_array = [
        gas_costs.builtins.pedersen,
        gas_costs.builtins.bitwise,
        gas_costs.builtins.ecop,
        gas_costs.builtins.poseidon,
        gas_costs.builtins.add_mod,
        gas_costs.builtins.mul_mod,
    ];

    let data = builtin_price_array
        .iter()
        .map(|&x| MaybeRelocatable::from(Felt::from(x)))
        .collect::<Vec<_>>();
    let builtin_cost_segment_start = read_only_segments.allocate(&mut runner.vm, &data)?;

    // Put a pointer to the builtin cost segment at the end of the program (after the
    // additional `ret` statement).
    let mut ptr = (runner.vm.get_pc() + bytecode_length)?;
    // Push a `ret` opcode.
    write_felt(
        &mut runner.vm,
        &mut ptr,
        Felt::from(0x208b_7fff_7fff_7ffe_u128),
    )?;
    // Push a pointer to the builtin cost segment.
    write_maybe_relocatable(&mut runner.vm, &mut ptr, builtin_cost_segment_start)?;

    let program_extra_data_length = 2;
    Ok(program_extra_data_length)
}

// TODO to be upstreamed to blockifer
pub fn finalize_runner(
    runner: &mut CairoRunner,
    n_total_args: usize,
    program_extra_data_length: usize,
) -> Result<(), PostExecutionError> {
    // Close memory holes in segments (OS code touches those memory cells, we simulate it).
    let program_start_ptr = runner
        .program_base
        .expect("The `program_base` field should be initialized after running the entry point.");
    let program_end_ptr = (program_start_ptr + runner.get_program().data_len())?;
    runner
        .vm
        .mark_address_range_as_accessed(program_end_ptr, program_extra_data_length)?;

    let initial_fp = runner
        .get_initial_fp()
        .expect("The `initial_fp` field should be initialized after running the entry point.");
    // When execution starts the stack holds the EP arguments + [ret_fp, ret_pc].
    let args_ptr = (initial_fp - (n_total_args + 2))?;
    runner
        .vm
        .mark_address_range_as_accessed(args_ptr, n_total_args)?;
    Ok(())
}

// TODO to be upstreamed to blockifer
pub fn extract_vm_resources(
    runner: &CairoRunner,
    syscall_handler: &SyscallHintProcessor<'_>,
    tracked_resource: TrackedResource,
) -> Result<ExecutionResources, PostExecutionError> {
    match tracked_resource {
        TrackedResource::CairoSteps => {
            // Take into account the resources of the current call, without inner calls.
            // Has to happen after marking holes in segments as accessed.
            let mut vm_resources_without_inner_calls = runner
                .get_execution_resources()
                .map_err(VirtualMachineError::RunnerError)?
                .filter_unused_builtins();
            let versioned_constants = syscall_handler.base.context.versioned_constants();
            if versioned_constants.segment_arena_cells {
                vm_resources_without_inner_calls
                    .builtin_instance_counter
                    .get_mut(&BuiltinName::segment_arena)
                    .map_or_else(|| {}, |val| *val *= SEGMENT_ARENA_BUILTIN_SIZE);
            }
            // Take into account the syscall resources of the current call.
            vm_resources_without_inner_calls += &versioned_constants
                .get_additional_os_syscall_resources(&syscall_handler.syscalls_usage);
            Ok(vm_resources_without_inner_calls)
        }
        TrackedResource::SierraGas => Ok(ExecutionResources::default()),
    }
}

// TODO to be upstreamed to blockifer
pub fn total_vm_resources(
    vm_resources_without_inner_calls: &ExecutionResources,
    inner_calls: &[CallInfo],
) -> ExecutionResources {
    vm_resources_without_inner_calls + &CallInfo::summarize_vm_resources(inner_calls.iter())
}

// TODO to be upstreamed to blockifer
#[allow(clippy::trivially_copy_pass_by_ref)]
// Reason copied: Required by `finalize_execution`
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
    Ok(CallResult {
        failed,
        retdata: read_execution_retdata(runner, retdata_size, retdata_start)?,
        gas_consumed,
    })
}
