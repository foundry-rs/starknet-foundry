use blockifier::abi::constants;
use blockifier::fee::fee_utils::calculate_l1_gas_by_vm_usage;

use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::context::TransactionContext;
use blockifier::fee::eth_gas_constants;
use blockifier::fee::gas_usage::{
    get_consumed_message_to_l2_emissions_cost, get_da_gas_cost,
    get_log_message_to_l1_emissions_cost,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::{GasVector, HasRelatedFeeType};
use blockifier::utils::u128_from_usize;
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

    let l1_gas_by_vm_costs =
        calculate_l1_gas_by_vm_usage(versioned_constants, &resources.execution_resources, 0)
            .expect("Could not calculate gas");

    let events_costs = get_events_cost(resources.events, transaction_context);

    let gas = l1_data_cost + l1_gas_by_vm_costs + messaging_gas_vector + events_costs;

    Ok(gas.l1_gas + gas.l1_data_gas)
}

fn get_events_cost(
    events: Vec<EventContent>,
    transaction_context: &TransactionContext,
) -> GasVector {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    // https://github.com/starkware-libs/blockifier/blob/9eceb37ab579844d8f00125edd82aeb235cd231e/crates/blockifier/src/transaction/objects.rs#L405
    let mut total_event_keys = 0;
    let mut total_event_data_size = 0;

    for event_content in events {
        // TODO(barak: 18/03/2024): Once we start charging per byte
        // change to num_bytes_keys
        // and num_bytes_data.
        total_event_data_size += u128_from_usize(event_content.data.0.len());
        total_event_keys += u128_from_usize(event_content.keys.len());
    }

    // https://github.com/starkware-libs/blockifier/blob/06abd15b92c8933116c7e5b4e8d1fd3936fe4b5f/crates/blockifier/src/transaction/objects.rs#L388
    let l2_resource_gas_costs = &versioned_constants.l2_resource_gas_costs;
    let (event_key_factor, data_word_cost) = (
        l2_resource_gas_costs.event_key_factor,
        l2_resource_gas_costs.gas_per_data_felt,
    );

    let l1_gas: u128 = (data_word_cost
        * (event_key_factor * total_event_keys + total_event_data_size))
        .to_integer();

    GasVector::from_l1_gas(l1_gas)
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
        ),
        l1_data_gas: 0,
    } + log_msg_to_l1_event_emission_cost
        + l1_handlers_emission_costs;

    let sharp_gas_usage = GasVector {
        l1_gas: u128_from_usize(
            message_segment_length * eth_gas_constants::SHARP_GAS_PER_MEMORY_WORD,
        ),
        l1_data_gas: 0,
    };

    starknet_gas_usage + sharp_gas_usage
}

fn get_l1_data_cost(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
) -> Result<GasVector, StateError> {
    let mut state_changes = state.get_actual_state_changes()?;
    // compiled_class_hash_updates is used only for keeping track of declares
    // which we don't want to include in gas cost
    state_changes.0.compiled_class_hashes.clear();

    let state_changes_count = state_changes.count_for_fee_charge(
        None,
        transaction_context
            .block_context
            .chain_info()
            .fee_token_address(&transaction_context.tx_info.fee_type()),
    );

    let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    let l1_data_gas_cost = get_da_gas_cost(&state_changes_count, use_kzg_da);
    Ok(l1_data_gas_cost)
}

pub fn check_available_gas(
    available_gas: Option<usize>,
    summary: TestCaseSummary<Single>,
) -> TestCaseSummary<Single> {
    match summary {
        TestCaseSummary::Passed {
            name,
            arguments,
            gas_info,
            ..
        } if available_gas.is_some_and(|available_gas| gas_info > available_gas as u128) => {
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
