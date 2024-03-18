use blockifier::execution::execution_utils::felt_to_stark_felt;
use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use cairo_felt::Felt252;
use cairo_vm::vm::trace::trace_entry::TraceEntry;
use cairo_vm::vm::vm_core::VirtualMachine;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

pub fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}

#[must_use]
pub fn sum_syscall_counters(mut a: SyscallCounter, b: &SyscallCounter) -> SyscallCounter {
    for (key, value) in b {
        *a.entry(*key).or_default() += *value;
    }
    a
}

#[must_use]
pub fn get_relocated_vm_trace(vm: &VirtualMachine) -> Vec<TraceEntry> {
    vm.get_relocated_trace()
        .unwrap()
        .iter()
        .map(|x| TraceEntry {
            pc: x.pc,
            ap: x.ap,
            fp: x.fp,
        })
        .collect()
}
