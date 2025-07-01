use blockifier::execution::syscalls::SyscallSelector;
use blockifier::execution::syscalls::hint_processor::SyscallUsageMap;
use blockifier::utils::u64_from_usize;
use blockifier::versioned_constants::VersionedConstants;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use starknet_api::transaction::fields::Calldata;
use starknet_types_core::felt::Felt;

#[must_use]
pub fn create_execute_calldata(calldata: &[Felt]) -> Calldata {
    Calldata(calldata.to_vec().into())
}

#[must_use]
pub fn sum_syscall_usage(mut a: SyscallUsageMap, b: &SyscallUsageMap) -> SyscallUsageMap {
    for (key, value) in b {
        a.entry(*key).or_default().call_count += value.call_count;
        a.entry(*key).or_default().linear_factor += value.linear_factor;
    }
    a
}

pub fn subtract_syscall_usage(mut a: SyscallUsageMap, b: &SyscallUsageMap) -> SyscallUsageMap {
    for (key, b_usage) in b {
        a.entry(*key).and_modify(|a_usage| {
            a_usage.call_count -= b_usage.call_count;
            a_usage.linear_factor -= b_usage.linear_factor;
        });
    }
    a.into_iter()
        .filter(|(_, usage)| usage.call_count > 0)
        .collect()
}

#[must_use]
pub fn get_syscalls_gas_consumed(
    syscall_usage: &SyscallUsageMap,
    versioned_constants: &VersionedConstants,
) -> u64 {
    syscall_usage
        .iter()
        .map(|(selector, usage)| {
            let syscall_gas_cost = versioned_constants.get_syscall_gas_cost(selector);

            // `linear_factor` is relevant only for `deploy` syscall, for other syscalls it is 0
            // `base_syscall_cost` makes an assert that `linear_factor` is 0
            // Hence to get base cost for `deploy` we use `get_syscall_cost` with 0 as `linear_length`
            // For other syscalls we use `base_syscall_cost`, which is also an additional check that `linear_factor` is always 0 then
            let base_cost = if let SyscallSelector::Deploy = selector {
                syscall_gas_cost.get_syscall_cost(0)
            } else {
                syscall_gas_cost.base_syscall_cost()
            };

            // We want to calculate `base_cost * call_count + linear_cost * linear_factor`
            // And it is achieved by calculating `base_cost * (call_count - 1) + base_cost + linear_cost * linear_factor`
            // There is a field name `linear_factor` used in both `SyscallUsage` and `SyscallGasCost`
            // In `get_syscall_cost` function `linear_length` parameter is calldata length, hence `SyscallUsage.linear_factor`
            u64_from_usize(usage.call_count - 1) * base_cost
                + syscall_gas_cost.get_syscall_cost(u64_from_usize(usage.linear_factor))
        })
        .sum()
}

#[must_use]
pub fn get_relocated_vm_trace(cairo_runner: &mut CairoRunner) -> Vec<RelocatedTraceEntry> {
    // if vm execution failed, the trace is not relocated so we need to relocate it
    if cairo_runner.relocated_trace.is_none() {
        cairo_runner
            .relocate(true)
            .expect("relocation should not fail");
    }
    cairo_runner
        .relocated_trace
        .clone()
        .expect("relocated trace should be present")
}
