use crate::test_case_summary::{Single, TestCaseSummary};
use anyhow::anyhow;
use blockifier::abi::constants;
use blockifier::context::TransactionContext;
use blockifier::execution::call_info::EventSummary;
use blockifier::fee::resources::{
    ArchivalDataResources, ComputationResources, MessageResources, StarknetResources,
    StateResources, TransactionResources,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::HasRelatedFeeType;
use blockifier::utils::u64_from_usize;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::forge_config_extension::config::RawAvailableGasConfig;
use cheatnet::state::ExtendedStateReader;
use shared::print::print_as_warning;
use starknet_api::execution_resources::{GasAmount, GasVector};
use starknet_api::transaction::EventContent;
use starknet_api::transaction::fields::GasVectorComputationMode;

pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    resources: UsedResources,
    is_strk_token_predeployed: bool,
) -> Result<GasVector, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();

    let message_resources = get_messages_resources(
        &resources.l2_to_l1_payload_lengths,
        &resources.l1_handler_payload_lengths,
    );

    let mut state_resources = get_state_resources(transaction_context, state)?;
    if is_strk_token_predeployed {
        state_resources =
            reduce_state_resources_after_strk_token_predeployment(&mut state_resources);
    }

    let archival_data_resources = get_archival_data_resources(resources.events);

    let starknet_resources = StarknetResources {
        archival_data: archival_data_resources,
        messages: message_resources,
        state: state_resources,
    };
    let computation_resources = ComputationResources {
        vm_resources: resources.execution_resources.clone(),
        n_reverted_steps: 0,
        sierra_gas: resources.gas_consumed,
        reverted_sierra_gas: GasAmount::ZERO,
    };

    let transaction_resources = TransactionResources {
        starknet_resources,
        computation: computation_resources,
    };

    let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    Ok(transaction_resources.to_gas_vector(
        versioned_constants,
        use_kzg_da,
        &GasVectorComputationMode::All,
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

    // calldata length, signature length and code size are set to 0, because
    // we don't include them in estimations
    // ref: https://github.com/foundry-rs/starknet-foundry/blob/5ce15b029135545452588c00aae580c05eb11ca8/docs/src/testing/gas-and-resource-estimation.md?plain=1#L73
    ArchivalDataResources {
        event_summary,
        calldata_length: 0,
        signature_length: 0,
        code_size: 0,
    }
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

fn reduce_state_resources_after_strk_token_predeployment(
    state_resources: &mut StateResources,
) -> StateResources {
    // STRK predeployment results in state changes. To avoid including them in gas cost
    // we need to reduce the state changes count. These calculations could be slightly inaccurate only if
    // someone would modify storage cells which are changed in the STRK constructor.

    state_resources
        .state_changes_for_fee
        .state_changes_count
        .n_storage_updates -= 10;
    state_resources
        .state_changes_for_fee
        .state_changes_count
        .n_class_hash_updates -= 1;
    state_resources
        .state_changes_for_fee
        .state_changes_count
        .n_modified_contracts -= 1;
    state_resources.state_changes_for_fee.n_allocated_keys -= 10;

    state_resources.clone()
}

pub fn check_available_gas(
    available_gas: Option<RawAvailableGasConfig>,
    summary: TestCaseSummary<Single>,
) -> TestCaseSummary<Single> {
    match summary {
        TestCaseSummary::Passed {
            name,
            arguments,
            gas_info,
            debugging_trace,
            ..
        } if available_gas.is_some_and(|available_gas| match available_gas {
            RawAvailableGasConfig::MaxGas(gas) => {
                // todo(3109): remove uunnamed argument in available_gas
                print_as_warning(&anyhow!(
                    "Setting available_gas with unnamed argument is deprecated. \
                Consider setting resource bounds (l1_gas, l1_data_gas and l2_gas) explicitly."
                ));
                // convert resource bounds to classic l1_gas using formula
                // l1_gas + l1_data_gas + (l2_gas / 40000)
                // because 100 l2_gas = 0.0025 l1_gas
                gas_info.l1_gas + gas_info.l1_data_gas + (gas_info.l2_gas / 40000)
                    > GasAmount(gas as u64)
            }
            RawAvailableGasConfig::MaxResourceBounds(bounds) => {
                let av_gas = bounds.to_gas_vector();
                gas_info.l1_gas > av_gas.l1_gas
                    || gas_info.l1_data_gas > av_gas.l1_data_gas
                    || gas_info.l2_gas > av_gas.l2_gas
            }
        }) =>
        {
            TestCaseSummary::Failed {
                name,
                msg: Some(format!(
                    "\n\tTest cost exceeded the available gas. Consumed l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{}",
                    gas_info.l1_gas, gas_info.l1_data_gas, gas_info.l2_gas
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
