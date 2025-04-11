use blockifier::execution::contract_class::EntryPointV1;
use blockifier::execution::entry_point::EntryPointExecutionResult;
use blockifier::execution::errors::EntryPointExecutionError;
use blockifier::execution::execution_utils::Args;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::builtin_runner::BuiltinRunner;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner};
use cairo_vm::vm::security::verify_secure_runner;
use num_traits::Zero;
use starknet_types_core::felt::Felt;

/// Module containing copied code to be upstreamed to blockifier

/// Why copied: Signature change
/// Runs the runner from the given PC.
pub(crate) fn run_entry_point(
    runner: &mut CairoRunner,
    // Modified code
    // hint_processor: &mut SyscallHintProcessor<'_>,
    hint_processor: &mut dyn HintProcessor,
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

/// Why copied: Required by `run_entry_point`
///
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
