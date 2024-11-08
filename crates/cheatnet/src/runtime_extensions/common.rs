use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use starknet_api::transaction::Calldata;
use starknet_types_core::felt::Felt;

#[must_use]
pub fn create_execute_calldata(calldata: &[Felt]) -> Calldata {
    Calldata(calldata.to_vec().into())
}

#[must_use]
pub fn sum_syscall_counters(mut a: SyscallCounter, b: &SyscallCounter) -> SyscallCounter {
    for (key, value) in b {
        *a.entry(*key).or_default() += *value;
    }
    a
}

#[must_use]
pub fn get_relocated_vm_trace(cairo_runner: &CairoRunner) -> Vec<RelocatedTraceEntry> {
    cairo_runner.relocated_trace.clone().unwrap()
}
