use blockifier::abi::constants;
use std::collections::HashMap;

use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::context::TransactionContext;
use blockifier::execution::call_info::OrderedEvent;
use blockifier::fee::eth_gas_constants;
use blockifier::fee::fee_utils::calculate_tx_gas_vector;
use blockifier::fee::gas_usage::{
    get_consumed_message_to_l2_emissions_cost, get_da_gas_cost, get_events_milligas_cost,
    get_log_message_to_l1_emissions_cost,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::{GasVector, HasRelatedFeeType, ResourcesMapping};
use blockifier::utils::{u128_from_usize, usize_from_u128};
use blockifier::versioned_constants::VersionedConstants;
use cairo_vm::vm::runners::builtin_runner::SEGMENT_ARENA_BUILTIN_NAME;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::state::ExtendedStateReader;
use starknet_api::transaction::EventContent;

pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: UsedResources,
) -> Result<u128, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    let messaging_gas_vector = get_messages_costs(
        &resources.l2_to_l1_payload_lengths,
        &resources.l1_handler_payload_lengths,
    );

    let l1_data_cost = get_l1_data_cost(transaction_context, state)?;
    let l1_and_vm_costs = get_l1_and_vm_costs(
        l1_data_cost,
        versioned_constants,
        resources.execution_resources,
    );

    let events_costs = get_events_cost(resources.events, transaction_context);

    let gas = l1_and_vm_costs + messaging_gas_vector + events_costs;

    Ok(gas.l1_gas + gas.l1_data_gas)
}

fn get_events_cost(
    events: Vec<EventContent>,
    transaction_context: &TransactionContext,
) -> GasVector {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    let ordered_events: Vec<OrderedEvent> = events
        .into_iter()
        .map(|content| OrderedEvent {
            order: 0, // Order does not matter here
            event: content,
        })
        .collect();
    let milligas = get_events_milligas_cost(&ordered_events, versioned_constants);
    GasVector {
        l1_gas: milligas / 1000_u128,
        l1_data_gas: 0_u128,
    }
}

// Put together from a few blockifier functions
// In a transaction (blockifier), there's only one l1_handler possible so we have to calculate those costs manually
// (it's not the case in a scope of the test)
fn get_messages_costs(
    l2_to_l1_payloads_lengths: &[usize],
    l1_handler_payloads_lengths: &[usize],
) -> GasVector {
    let l2_to_l1_segment_length = l2_to_l1_payloads_lengths
        .iter()
        .map(|payload_length| constants::L2_TO_L1_MSG_HEADER_SIZE + payload_length)
        .sum::<usize>();

    let l1_to_l2_segment_length = l1_handler_payloads_lengths
        .iter()
        .map(|payload_length| constants::L1_TO_L2_MSG_HEADER_SIZE + payload_length)
        .sum::<usize>();
    let message_segment_length = l2_to_l1_segment_length + l1_to_l2_segment_length;

    let n_l2_to_l1_messages = l2_to_l1_payloads_lengths.len();
    let n_l1_to_l2_messages = l1_handler_payloads_lengths.len();

    let l1_handlers_emission_costs = l1_handler_payloads_lengths
        .iter()
        .map(|l1_handler_payload_length| {
            get_consumed_message_to_l2_emissions_cost(Some(*l1_handler_payload_length))
        })
        .sum();

    let log_msg_to_l1_event_emission_cost =
        get_log_message_to_l1_emissions_cost(l2_to_l1_payloads_lengths);
    let starknet_gas_usage = GasVector {
        l1_gas: u128_from_usize(
            message_segment_length * eth_gas_constants::GAS_PER_MEMORY_WORD
                + n_l2_to_l1_messages * eth_gas_constants::GAS_PER_ZERO_TO_NONZERO_STORAGE_SET
                + n_l1_to_l2_messages * eth_gas_constants::GAS_PER_COUNTER_DECREASE,
        )
        .expect("Could not convert starknet gas usage from usize to u128."),
        l1_data_gas: 0,
    } + log_msg_to_l1_event_emission_cost
        + l1_handlers_emission_costs;

    let sharp_gas_usage = GasVector {
        l1_gas: u128_from_usize(
            message_segment_length * eth_gas_constants::SHARP_GAS_PER_MEMORY_WORD,
        )
        .expect("Could not convert sharp gas usage from usize to u128."),
        l1_data_gas: 0,
    };

    starknet_gas_usage + sharp_gas_usage
}

fn get_l1_and_vm_costs(
    l1_data_costs: GasVector,
    versioned_constants: &VersionedConstants,
    execution_resources: ExecutionResources,
) -> GasVector {
    let resources_mapping = get_resources_mapping(l1_data_costs, execution_resources);

    calculate_tx_gas_vector(&resources_mapping, versioned_constants)
        .expect("Could not calculate gas")
}

fn get_l1_data_cost(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
) -> Result<GasVector, StateError> {
    let mut state_changes = state.get_actual_state_changes()?;
    // compiled_class_hash_updates is used only for keeping track of declares
    // which we don't want to include in gas cost
    state_changes.compiled_class_hash_updates.clear();

    let state_changes_count = state_changes.count_for_fee_charge(
        None,
        transaction_context
            .block_context
            .chain_info()
            .fee_token_address(&transaction_context.tx_info.fee_type()),
    );

    let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    let l1_data_gas_cost = get_da_gas_cost(state_changes_count, use_kzg_da);
    Ok(l1_data_gas_cost)
}

fn get_resources_mapping(
    data_availability_gas_cost: GasVector,
    mut vm_usage: ExecutionResources,
) -> ResourcesMapping {
    let l1_gas_usage = usize_from_u128(data_availability_gas_cost.l1_gas)
        .expect("tx_gas_cost.l1_gas conversion should not fail as the value is a converted usize.");
    let l1_blob_gas_usage = usize_from_u128(data_availability_gas_cost.l1_data_gas).expect(
        "tx_gas_cost.l1_data_gas conversion should not fail as the value is a converted usize.",
    );
    // An estimation of what segment arena consumes
    let n_steps = vm_usage.n_steps
        + 10 * vm_usage
            .builtin_instance_counter
            .remove(SEGMENT_ARENA_BUILTIN_NAME)
            .unwrap_or_default();

    let mut tx_resources = HashMap::from([
        (constants::L1_GAS_USAGE.to_string(), l1_gas_usage),
        (constants::BLOB_GAS_USAGE.to_string(), l1_blob_gas_usage),
        (
            constants::N_STEPS_RESOURCE.to_string(),
            n_steps + vm_usage.n_memory_holes,
        ),
    ]);
    tx_resources.extend(vm_usage.builtin_instance_counter);
    ResourcesMapping(tx_resources)
}

pub fn check_available_gas(
    available_gas: &Option<usize>,
    summary: TestCaseSummary<Single>,
) -> TestCaseSummary<Single> {
    match summary {
        TestCaseSummary::Passed {
            name,
            arguments,
            gas_info,
            ..
        } if available_gas.map_or(false, |available_gas| gas_info > available_gas as u128) => {
            TestCaseSummary::Failed {
                name,
                msg: Some(format!(
                    "\n\tTest cost exceeded the available gas. Consumed gas: ~{gas_info}"
                )),
                arguments,
                test_statistics: (),
            }
        }
        _ => summary,
    }
}
