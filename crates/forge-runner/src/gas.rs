use std::collections::HashMap;

use blockifier::fee::eth_gas_constants;
use blockifier::fee::fee_utils::calculate_tx_l1_gas_usage;
use blockifier::fee::os_resources::OS_RESOURCES;
use blockifier::fee::os_usage::get_additional_os_resources;
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::{
    abi::constants, block_context::BlockContext, execution::entry_point::ExecutionResources,
    fee::fee_utils::calculate_l1_gas_by_vm_usage, transaction::objects::ResourcesMapping,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;

#[must_use]
pub fn gas_from_execution_resources(
    block_context: &BlockContext,
    resources: &ExecutionResources,
    onchain_data_segment_len: usize,
) -> u128 {
    let total_vm_usage = get_total_vm_usage(resources);
    let resource_mapping =
        vm_execution_resources_to_resource_mapping(&total_vm_usage, onchain_data_segment_len);
    calculate_tx_l1_gas_usage(&resource_mapping, block_context)
        .expect("Calculating gas failed, some resources were not included.")
}

#[must_use]
fn vm_execution_resources_to_resource_mapping(
    execution_resources: &VmExecutionResources,
    onchain_data_segment_len: usize,
) -> ResourcesMapping {
    let mut map = HashMap::from([
        (
            constants::N_STEPS_RESOURCE.to_string(),
            execution_resources.n_steps,
        ),
        (
            constants::GAS_USAGE.to_string(),
            onchain_data_segment_len * eth_gas_constants::SHARP_GAS_PER_MEMORY_WORD,
        ),
    ]);
    map.extend(execution_resources.builtin_instance_counter.clone());
    ResourcesMapping(map)
}

/// `total_vm_usage` consists of resources used by vm (`vm_resources`)
/// and additional resources computed from used syscalls (`get_additional_os_resources`).
/// Unfortunately `get_additional_os_resources` function adds resources used by os,
/// so we have to subtract them
fn get_total_vm_usage(resources: &ExecutionResources) -> VmExecutionResources {
    let unnecessary_added_resources = OS_RESOURCES
        .execute_txs_inner()
        .get(&TransactionType::InvokeFunction)
        .expect("`OS_RESOURCES` must contain all transaction types.");

    let total_vm_usage = &resources.vm_resources
        + &(&get_additional_os_resources(
            &resources.syscall_counter,
            TransactionType::InvokeFunction,
        )
        .unwrap()
            - unnecessary_added_resources);
    total_vm_usage.filter_unused_builtins()
}
