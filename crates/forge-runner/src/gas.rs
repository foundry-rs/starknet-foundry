use crate::forge_config::ForgeTrackedResource;
use crate::gas::resources::GasCalculationResources;
use crate::test_case_summary::{Single, TestCaseSummary};
use blockifier::context::TransactionContext;
use blockifier::fee::resources::{StarknetResources, StateResources, TransactionResources};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::HasRelatedFeeType;
use cheatnet::runtime_extensions::forge_config_extension::config::{
    RawAvailableGasConfig, RawAvailableResourceBoundsConfig,
};
use cheatnet::runtime_extensions::outer_call_runtime_extension::rpc::UsedResources;
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
    used_resources: &UsedResources,
) -> Result<GasVector, StateError> {
    let versioned_constants = transaction_context.block_context.versioned_constants();
    let resources = GasCalculationResources::from_used_resources(used_resources);

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
    available_gas: Option<RawAvailableGasConfig>,
    summary: TestCaseSummary<Single>,
    tracked_resource: ForgeTrackedResource,
) -> TestCaseSummary<Single> {
    match summary {
        TestCaseSummary::Passed {
            name,
            msg,
            gas_info,
            used_resources,
            test_statistics,
            debugging_trace,
            trace_data,
        } => {
            let failure_message = available_gas.and_then(|available_gas| {
                check_available_gas_limit(
                    available_gas,
                    gas_info.gas_used,
                    &used_resources,
                    tracked_resource,
                )
            });

            if let Some(failure_message) = failure_message {
                TestCaseSummary::Failed {
                    name,
                    msg: Some(failure_message),
                    fuzzer_args: Vec::default(),
                    test_statistics: (),
                    debugging_trace,
                }
            } else {
                TestCaseSummary::Passed {
                    name,
                    msg,
                    debugging_trace,
                    gas_info,
                    used_resources,
                    test_statistics,
                    trace_data,
                }
            }
        }
        _ => summary,
    }
}

fn check_available_gas_limit(
    available_gas: RawAvailableGasConfig,
    gas_used: GasVector,
    used_resources: &UsedResources,
    tracked_resource: ForgeTrackedResource,
) -> Option<String> {
    match available_gas {
        // Sierra gas limit cannot be used when tracked resource is Cairo steps.
        RawAvailableGasConfig::MaxSierraGas(_)
            if tracked_resource == ForgeTrackedResource::CairoSteps =>
        {
            Some(
                "\n\tSetting a Sierra gas limit via `#[available_gas]` requires running the test with Sierra gas tracking, but it is run with Cairo steps tracking. Use resource bounds (`l1_gas`, `l1_data_gas`, `l2_gas`) instead, or run with Sierra gas tracking."
                    .to_string(),
            )
        }
        RawAvailableGasConfig::MaxSierraGas(max_gas) => {
            check_sierra_gas_limit(max_gas, used_resources)
        }
        RawAvailableGasConfig::MaxResourceBounds(available_resource_bounds) => {
            check_resource_bounds(&available_resource_bounds, gas_used)
        }
    }
}

fn check_sierra_gas_limit(max_gas: usize, used_resources: &UsedResources) -> Option<String> {
    let gas_consumed = used_resources
        .execution_summary
        .charged_resources
        .gas_consumed
        .0;

    (gas_consumed > max_gas as u64).then(|| {
        format!(
            "\n\tTest cost exceeded the available sierra gas. Consumed sierra_gas: ~{gas_consumed}, available: {max_gas}"
        )
    })
}

fn check_resource_bounds(
    available_resource_bounds: &RawAvailableResourceBoundsConfig,
    gas_used: GasVector,
) -> Option<String> {
    let av_gas = available_resource_bounds.to_gas_vector();

    (gas_used.l1_gas > av_gas.l1_gas
        || gas_used.l1_data_gas > av_gas.l1_data_gas
        || gas_used.l2_gas > av_gas.l2_gas)
        .then(|| {
            format!(
                "\n\tTest cost exceeded the available gas. Consumed l1_gas: ~{}, l1_data_gas: ~{}, l2_gas: ~{}",
                gas_used.l1_gas, gas_used.l1_data_gas, gas_used.l2_gas
            )
        })
}
