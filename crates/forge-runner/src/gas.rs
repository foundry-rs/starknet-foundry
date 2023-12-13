use std::collections::HashMap;

use blockifier::fee::eth_gas_constants;
use blockifier::fee::fee_utils::calculate_tx_l1_gas_usage;
use blockifier::fee::gas_usage::get_message_segment_length;
use blockifier::fee::os_resources::OS_RESOURCES;
use blockifier::fee::os_usage::get_additional_os_resources;
use blockifier::state::cached_state::{CachedState, StateChangesCount};
use blockifier::transaction::transaction_types::TransactionType;
use blockifier::{
    abi::constants, block_context::BlockContext, execution::entry_point::ExecutionResources,
    transaction::objects::ResourcesMapping,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::state::ExtendedStateReader;

#[must_use]
pub fn calculate_used_gas(
    block_context: &BlockContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: &UsedResources,
) -> u128 {
    let total_vm_usage = get_total_vm_usage(&resources.execution_resources);
    let mut state_changes = state
        .get_actual_state_changes_for_fee_charge(
            block_context.fee_token_addresses.eth_fee_token_address,
            None,
        )
        .unwrap();
    // compiled_class_hash_updates is used only for keeping track of declares
    // which we don't want to include in gas cost
    state_changes.compiled_class_hash_updates.clear();

    let l1_gas_usage = get_l1_gas_usage(
        &resources.l2_to_l1_payloads_length,
        StateChangesCount::from(&state_changes),
    );

    let resource_mapping = used_resources_to_resource_mapping(&total_vm_usage, l1_gas_usage);
    calculate_tx_l1_gas_usage(&resource_mapping, block_context)
        .expect("Calculating gas failed, some resources were not included.")
}

#[must_use]
fn used_resources_to_resource_mapping(
    execution_resources: &VmExecutionResources,
    l1_gas_usage: usize,
) -> ResourcesMapping {
    let mut map = HashMap::from([
        (
            constants::N_STEPS_RESOURCE.to_string(),
            execution_resources.n_steps,
        ),
        (constants::GAS_USAGE.to_string(), l1_gas_usage),
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

fn get_l1_gas_usage(
    l2_to_l1_payloads_length: &[usize],
    state_changes_count: StateChangesCount,
) -> usize {
    let message_segment_length = get_message_segment_length(l2_to_l1_payloads_length, None);
    let onchain_data_segment_length = get_onchain_data_segment_length(state_changes_count);

    message_segment_length * eth_gas_constants::SHARP_GAS_PER_MEMORY_WORD
        + onchain_data_segment_length * eth_gas_constants::SHARP_GAS_PER_MEMORY_WORD
}

// TODO::copied from blockifier, because it became private
fn get_onchain_data_segment_length(state_changes_count: StateChangesCount) -> usize {
    // For each newly modified contract:
    // contract address (1 word).
    // + 1 word with the following info: A flag indicating whether the class hash was updated, the
    // number of entry updates, and the new nonce.
    let mut onchain_data_segment_length = state_changes_count.n_modified_contracts * 2;
    // For each class updated (through a deploy or a class replacement).
    onchain_data_segment_length +=
        state_changes_count.n_class_hash_updates * constants::CLASS_UPDATE_SIZE;
    // For each modified storage cell: key, new value.
    onchain_data_segment_length += state_changes_count.n_storage_updates * 2;
    // For each compiled class updated (through declare): class_hash, compiled_class_hash
    onchain_data_segment_length += state_changes_count.n_compiled_class_hash_updates * 2;

    onchain_data_segment_length
}
