use blockifier::abi::constants;
use std::collections::HashMap;

use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::context::TransactionContext;
use blockifier::fee::eth_gas_constants;
use blockifier::fee::fee_utils::calculate_tx_gas_vector;
use blockifier::fee::gas_usage::{
    get_consumed_message_to_l2_emissions_cost, get_da_gas_cost,
    get_log_message_to_l1_emissions_cost,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::{GasVector, HasRelatedFeeType, ResourcesMapping};
use blockifier::utils::{u128_from_usize, usize_from_u128};
use cairo_vm::vm::runners::builtin_runner::SEGMENT_ARENA_BUILTIN_NAME;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::state::ExtendedStateReader;

// Put together from a few blockifier functions
fn calculate_messages_costs(
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
    let starknet_gas_usage = GasVector {
        l1_gas: u128_from_usize(
            message_segment_length * eth_gas_constants::GAS_PER_MEMORY_WORD
                + n_l2_to_l1_messages * eth_gas_constants::GAS_PER_ZERO_TO_NONZERO_STORAGE_SET
                + n_l1_to_l2_messages * eth_gas_constants::GAS_PER_COUNTER_DECREASE,
        )
        .expect("Could not convert starknet gas usage from usize to u128."),
        l1_data_gas: 0,
    } + get_log_message_to_l1_emissions_cost(l2_to_l1_payloads_lengths)
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

pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: &UsedResources,
) -> Result<u128, StateError> {
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

    let tx_gas_cost = get_da_gas_cost(state_changes_count, false);
    // TODO (1796): This should probably be added to the total cost estimate
    // + get_tx_events_gas_cost(call_infos, versioned_constants);

    let l1_gas_usage = usize_from_u128(tx_gas_cost.l1_gas)
        .expect("tx_gas_cost.l1_gas conversion should not fail as the value is a converted usize.");
    let l1_blob_gas_usage = usize_from_u128(tx_gas_cost.l1_data_gas).expect(
        "tx_gas_cost.l1_data_gas conversion should not fail as the value is a converted usize.",
    );

    let versioned_constants = transaction_context.block_context.versioned_constants();
    let mut total_vm_usage = resources.execution_resources.filter_unused_builtins();
    total_vm_usage += &versioned_constants
        .get_additional_os_syscall_resources(&resources.syscall_counter)
        .expect("Could not get additional costs");

    // An estimation of what segment arena consumes
    let n_steps = total_vm_usage.n_steps
        + 10 * total_vm_usage
            .builtin_instance_counter
            .remove(SEGMENT_ARENA_BUILTIN_NAME)
            .unwrap_or_default();

    let mut tx_resources = HashMap::from([
        (constants::L1_GAS_USAGE.to_string(), l1_gas_usage),
        (constants::BLOB_GAS_USAGE.to_string(), l1_blob_gas_usage),
        (
            constants::N_STEPS_RESOURCE.to_string(),
            n_steps + total_vm_usage.n_memory_holes,
        ),
    ]);
    tx_resources.extend(total_vm_usage.builtin_instance_counter);

    let mut gas = calculate_tx_gas_vector(&ResourcesMapping(tx_resources), versioned_constants)
        .expect("Could not calculate gas");
    let messaging_costs = calculate_messages_costs(
        &resources.l2_to_l1_payloads_lengths,
        &resources.l1_handler_payloads_lengths,
    );

    gas = gas + messaging_costs;

    Ok(gas.l1_gas)
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
