use crate::gas::resources::GasCalculationResources;
use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::context::TransactionContext;
use blockifier::fee::resources::{StarknetResources, StateResources, TransactionResources};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::HasRelatedFeeType;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::forge_config_extension::config::RawAvailableResourceBoundsConfig;
use cheatnet::state::ExtendedStateReader;
use starknet_api::execution_resources::GasVector;
use starknet_api::transaction::fields::GasVectorComputationMode;

pub mod report;
pub mod resources;
pub mod stats;
mod utils;

#[tracing::instrument(skip_all, level = "debug")]
pub fn calculate_used_gas(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
    used_resources: UsedResources,
) -> Result<GasVector, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();
    let resources = GasCalculationResources::from_used_resources(&used_resources);

    let starknet_resources = StarknetResources {
        archival_data: resources.to_archival_resources(),
        messages: resources.to_message_resources(),
        state: get_state_resources(transaction_context, state)?,
    };

    let transaction_resources = TransactionResources {
        starknet_resources,
        computation: resources.to_computation_resources(),
    };

    let use_kzg_da = transaction_context.block_context.block_info().use_kzg_da;
    Ok(transaction_resources.to_gas_vector(
        versioned_constants,
        use_kzg_da,
        &GasVectorComputationMode::All,
    ))
}

fn get_state_resources(
    transaction_context: &TransactionContext,
    state: &mut CachedState<ExtendedStateReader>,
) -> Result<StateResources, StateError> {
    let mut state_changes = state.to_state_diff()?;
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
    available_gas: Option<RawAvailableResourceBoundsConfig>,
    summary: TestCaseSummary<Single>,
) -> TestCaseSummary<Single> {
    match summary {
        TestCaseSummary::Passed {
            name,
            gas_info,
            debugging_trace,
            ..
        } if available_gas.is_some_and(|available_gas| {
            let av_gas = available_gas.to_gas_vector();
            gas_info.gas_used.l1_gas > av_gas.l1_gas
                || gas_info.gas_used.l1_data_gas > av_gas.l1_data_gas
                || gas_info.gas_used.l2_gas > av_gas.l2_gas
        }) =>
        {
            TestCaseSummary::Failed {
                name,
                msg: Some(format!(
                    "\n\tTest cost exceeded the available gas. Consumed l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{}",
                    gas_info.gas_used.l1_gas,
                    gas_info.gas_used.l1_data_gas,
                    gas_info.gas_used.l2_gas
                )),
                fuzzer_args: Vec::default(),
                test_statistics: (),
                debugging_trace,
            }
        }
        _ => summary,
    }
}
