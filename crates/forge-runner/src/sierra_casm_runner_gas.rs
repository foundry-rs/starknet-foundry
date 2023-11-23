use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::vm_core::VirtualMachine;

// similar to `finalize_execution` from blockifier
pub fn finalize(
    vm: &mut VirtualMachine,
    runner: &CairoRunner,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    n_total_args: usize,
    program_extra_data_length: usize,
) {
    let program_start_ptr = runner
        .program_base
        .expect("The `program_base` field should be initialized after running the entry point.");
    let program_end_ptr = (program_start_ptr + runner.get_program().data_len()).unwrap();
    vm.mark_address_range_as_accessed(program_end_ptr, program_extra_data_length)
        .unwrap();

    let initial_fp = runner
        .get_initial_fp()
        .expect("The `initial_fp` field should be initialized after running the entry point.");
    // When execution starts the stack holds the EP arguments + [ret_fp, ret_pc].
    let args_ptr = (initial_fp - (n_total_args + 2)).unwrap();
    vm.mark_address_range_as_accessed(args_ptr, n_total_args)
        .unwrap();
    syscall_handler
        .read_only_segments
        .mark_as_accessed(vm)
        .unwrap();

    let vm_resources_without_inner_calls = runner
        .get_execution_resources(vm)
        .map_err(VirtualMachineError::TracerError)
        .unwrap()
        .filter_unused_builtins();
    syscall_handler.resources.vm_resources += &vm_resources_without_inner_calls;
}
