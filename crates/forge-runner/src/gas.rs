use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::abi::constants;
use blockifier::context::TransactionContext;
use blockifier::execution::call_info::{ChargedResources, EventSummary, ExecutionSummary};
use blockifier::fee::resources::{
    ArchivalDataResources, ComputationResources, MessageResources, StarknetResources,
    StateResources, TransactionResources,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::HasRelatedFeeType;
use blockifier::utils::u64_from_usize;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::state::ExtendedStateReader;
use starknet_api::execution_resources::{GasAmount, GasVector};
use starknet_api::transaction::EventContent;
use starknet_api::transaction::fields::GasVectorComputationMode;
use std::collections::HashSet;

pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: UsedResources,
) -> Result<GasVector, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    let message_resources = get_messages_resources(
        &resources.l2_to_l1_payload_lengths,
        &resources.l1_handler_payload_lengths,
    );

    let state_resources = get_state_resources(transaction_context, state)?;

    let archival_data_resources = get_archival_data_resources(resources.events);

    let starknet_resources = StarknetResources {
        archival_data: archival_data_resources,
        messages: message_resources,
        state: state_resources,
    };
    let computation_resources = ComputationResources {
        vm_resources: resources.execution_resources.clone(),
        n_reverted_steps: 0,
        // TODO(#2977)
        sierra_gas: GasAmount(0),
        // TODO(#2977)
        reverted_sierra_gas: GasAmount(0),
    };

    let transaction_resources = TransactionResources {
        starknet_resources,
        computation: computation_resources,
    };

    let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    Ok(transaction_resources.to_gas_vector(
        versioned_constants,
        use_kzg_da,
        // TODO(#2977)
        &GasVectorComputationMode::NoL2Gas,
    ))
}

fn get_archival_data_resources(events: Vec<EventContent>) -> ArchivalDataResources {
    // Based on from https://github.com/starkware-libs/sequencer/blob/fc0f06a07f3338ae1e11612dcaed9c59373bca37/crates/blockifier/src/execution/call_info.rs#L222
    let mut event_summary = EventSummary {
        n_events: events.len(),
        ..Default::default()
    };
    for event in events {
        event_summary.total_event_data_size += u64_from_usize(event.data.0.len());
        event_summary.total_event_keys += u64_from_usize(event.keys.len());
    }

    // TODO(#2978) this is a workaround because we cannot create `ArchivalDataResources` directly yet
    //  because of private fields
    let dummy_execution_summary = ExecutionSummary {
        charged_resources: ChargedResources::default(),
        executed_class_hashes: HashSet::default(),
        visited_storage_entries: HashSet::default(),
        l2_to_l1_payload_lengths: vec![],
        event_summary,
    };

    let dummy_starknet_resources = StarknetResources::new(
        // calldata length, signature length and code size are set to 0, because
        // we don't include them in estimations
        // ref: https://github.com/foundry-rs/starknet-foundry/blob/5ce15b029135545452588c00aae580c05eb11ca8/docs/src/testing/gas-and-resource-estimation.md?plain=1#L73
        0,
        0,
        0,
        StateResources::default(),
        None,
        dummy_execution_summary,
    );

    dummy_starknet_resources.archival_data
}

// Put together from a few blockifier functions
// In a transaction (blockifier), there's only one l1_handler possible so we have to calculate those costs manually
// (it's not the case in a scope of the test)
fn get_messages_resources(
    l2_to_l1_payloads_lengths: &[usize],
    l1_handler_payloads_lengths: &[usize],
) -> MessageResources {
    let l2_to_l1_segment_length = l2_to_l1_payloads_lengths
        .iter()
        .map(|payload_length| constants::L2_TO_L1_MSG_HEADER_SIZE + payload_length)
        .sum::<usize>();

    let l1_to_l2_segment_length = l1_handler_payloads_lengths
        .iter()
        .map(|payload_length| constants::L1_TO_L2_MSG_HEADER_SIZE + payload_length)
        .sum::<usize>();
    let message_segment_length = l2_to_l1_segment_length + l1_to_l2_segment_length;

    MessageResources {
        l2_to_l1_payload_lengths: l2_to_l1_payloads_lengths.to_vec(),
        message_segment_length,
        // The logic for calculating gas vector treats `l1_handler_payload_size` being `Some`
        // as indication that L1 handler was used and adds gas cost for that.
        //
        // We need to set it to `None` if length is 0 to avoid including this extra cost.
        l1_handler_payload_size: if l1_to_l2_segment_length > 0 {
            Some(l1_to_l2_segment_length)
        } else {
            None
        },
    }
}

fn get_state_resources(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
) -> Result<StateResources, StateError> {
    let mut state_changes = state.get_actual_state_changes()?;
    // compiled_class_hash_updates is used only for keeping track of declares
    // which we don't want to include in gas cost
    state_changes.state_maps.compiled_class_hashes.clear();
    state_changes.state_maps.declared_contracts.clear();

    let state_changes_count = state_changes.count_for_fee_charge(
        None,
        transaction_context
            .block_context
            .chain_info()
            .fee_token_address(&transaction_context.tx_info.fee_type()),
    );

    Ok(StateResources {
        state_changes_for_fee: state_changes_count,
    })
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
            debugging_trace,
            ..
        } if available_gas.is_some_and(|available_gas| gas_info > available_gas as u128) => {
            TestCaseSummary::Failed {
                name,
                msg: Some(format!(
                    "\n\tTest cost exceeded the available gas. Consumed gas: ~{gas_info}"
                )),
                arguments,
                fuzzer_args: Vec::default(),
                test_statistics: (),
                debugging_trace,
            }
        }
        _ => summary,
    }
}
