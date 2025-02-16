use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::abi::constants;
use blockifier::context::TransactionContext;
use blockifier::execution::call_info::{EventSummary, ExecutionSummary};
use blockifier::fee::resources::{
    ArchivalDataResources, ComputationResources, MessageResources, StarknetResources,
    StateResources, TransactionResources,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::HasRelatedFeeType;
use blockifier::utils::u64_from_usize;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::state::ExtendedStateReader;
use starknet_api::execution_resources::{GasAmount, GasVector};
use starknet_api::transaction::fields::Calldata;
use starknet_api::transaction::EventContent;

pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: UsedResources,
    code_size: usize,
    calldata: Calldata,
) -> Result<GasVector, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();
    let message_resources = get_messages_resources(
        &resources.l2_to_l1_payload_lengths,
        &resources.l1_handler_payload_lengths,
    );

    let state_resources = get_state_resources(transaction_context, state)?;

    let archival_data_resources = get_archival_data_resources(
        resources.events,
        transaction_context,
        code_size,
        &calldata,
        state_resources.clone(),
    );

    dbg!(&resources.execution_resources);

    let starknet_resources = StarknetResources {
        archival_data: archival_data_resources,
        messages: message_resources,
        state: state_resources,
    };
    let sierra_gas = GasAmount::from(calculate_sierra_gas(&resources.execution_resources) as u64);
    let computation_resources = ComputationResources {
        vm_resources: resources.execution_resources.clone(),
        n_reverted_steps: 0,
        sierra_gas,
        // FIXME correct value
        reverted_sierra_gas: Default::default(),
    };

    let transaction_resources = TransactionResources {
        starknet_resources,
        computation: computation_resources,
    };

    dbg!(&transaction_resources);

    Ok(transaction_resources.to_gas_vector(
        versioned_constants,
        transaction_context.block_context.block_info().use_kzg_da,
        &transaction_context.get_gas_vector_computation_mode(),
    ))
}

fn get_archival_data_resources(
    events: Vec<EventContent>,
    transaction_context: &TransactionContext,
    code_size: usize,
    calldata: &Calldata,
    state_resources: StateResources,
) -> ArchivalDataResources {
    // FIXME link source
    let mut total_event_keys = 0;
    let mut total_event_data_size = 0;
    let n_events = events.len();

    for event_content in events {
        // TODO(barak: 18/03/2024): Once we start charging per byte
        // change to num_bytes_keys
        // and num_bytes_data.
        total_event_data_size += u64_from_usize(event_content.data.0.len());
        total_event_keys += u64_from_usize(event_content.keys.len());
    }

    // FIXME this is a workaround because we cannot create `ArchivalDataResources` directly yet
    //  because of private fields
    let dummy_execution_summary = ExecutionSummary {
        charged_resources: Default::default(),
        executed_class_hashes: Default::default(),
        visited_storage_entries: Default::default(),
        l2_to_l1_payload_lengths: vec![],
        event_summary: EventSummary {
            n_events,
            total_event_keys,
            total_event_data_size,
        },
    };

    let signature_length = transaction_context.tx_info.signature().0.len();
    let calldata_length = calldata.0.len();
    let dummy_starknet_resources = StarknetResources::new(
        calldata_length,
        signature_length,
        code_size,
        state_resources,
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

    // let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    // let l1_data_gas_cost = get_da_gas_cost(&state_changes_count, use_kzg_da);
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

fn calculate_sierra_gas(execution_resources: &ExecutionResources) -> usize {
    const COST_PER_CAIRO_STEP: usize = 100;
    const COST_PER_MEMORY_HOLE: usize = 50;

    let gas_from_steps = execution_resources.n_steps * COST_PER_CAIRO_STEP;

    let gas_from_memory_holes = execution_resources.n_memory_holes * COST_PER_MEMORY_HOLE;

    let gas_from_builtins: usize = execution_resources
        .builtin_instance_counter
        .iter()
        .map(|(libfunc, count)| calculate_libfunc_cost(*libfunc, *count))
        .sum();

    gas_from_steps + gas_from_memory_holes + gas_from_builtins
}

fn calculate_libfunc_cost(libfunc: BuiltinName, count: usize) -> usize {
    let cost_per_builtin = match libfunc {
        BuiltinName::output => 10, // FIXME: Not defined in costs table
        BuiltinName::range_check => 70,
        BuiltinName::pedersen => 4050,
        BuiltinName::poseidon => 491,
        BuiltinName::bitwise => 583,
        BuiltinName::ec_op => 4085,
        BuiltinName::add_mod => 230,
        BuiltinName::mul_mod => 604,
        BuiltinName::ecdsa => 5000,  // FIXME: Not defined in costs table
        BuiltinName::keccak => 3000, // FIXME: Not defined in costs table
        BuiltinName::segment_arena => 50, // FIXME: Not defined in costs table
        BuiltinName::range_check96 => 70, // FIXME: Not defined in costs table
    };
    count * cost_per_builtin
}
