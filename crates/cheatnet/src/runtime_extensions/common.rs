use blockifier::execution::execution_utils::felt_to_stark_felt;
use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use cairo_felt::Felt252;
use conversions::IntoConv;
use starknet_api::core::EntryPointSelector;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::Calldata;

pub fn create_execute_calldata(calldata: &[Felt252]) -> Calldata {
    let calldata: Vec<StarkFelt> = calldata.iter().map(felt_to_stark_felt).collect();
    Calldata(calldata.into())
}

#[must_use]
pub fn create_entry_point_selector(entry_point_selector: &Felt252) -> EntryPointSelector {
    EntryPointSelector(entry_point_selector.clone().into_()) //FIXME: remove clone and take `entry_point_selector` by value
}

#[must_use]
pub fn sum_syscall_counters(mut a: SyscallCounter, b: &SyscallCounter) -> SyscallCounter {
    for (key, value) in b {
        *a.entry(*key).or_default() += *value;
    }
    a
}
