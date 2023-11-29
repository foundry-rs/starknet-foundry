use std::collections::HashMap;

use blockifier::fee::eth_gas_constants;
use blockifier::fee::fee_utils::calculate_tx_l1_gas_usage;
use blockifier::fee::gas_usage::get_onchain_data_segment_length;
use blockifier::fee::os_resources::OS_RESOURCES;
use blockifier::fee::os_usage::get_additional_os_resources;
use blockifier::state::cached_state::{CachedState, StateChangesCount};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::{
    abi::constants, block_context::BlockContext, execution::entry_point::ExecutionResources,
    transaction::objects::ResourcesMapping,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
use cheatnet::state::ExtendedStateReader;

#[must_use]
pub fn calculate_used_gas(
    block_context: &BlockContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: &ExecutionResources,
) -> u128 {
    let total_vm_usage = get_total_vm_usage(resources);
    let onchain_data_segment_len = get_onchain_data_segment_length(StateChangesCount::from(
        &state
            .get_actual_state_changes_for_fee_charge(
                block_context.fee_token_addresses.eth_fee_token_address,
                None,
            )
            .unwrap(),
    ));

    let resource_mapping =
        used_resources_to_resource_mapping(&total_vm_usage, onchain_data_segment_len);
    calculate_tx_l1_gas_usage(&resource_mapping, block_context)
        .expect("Calculating gas failed, some resources were not included.")
}

#[must_use]
fn used_resources_to_resource_mapping(
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
