use crate::common::state::create_cached_state;
use crate::common::{call_contract_raw, deploy_contract, selector_from_name};
use blockifier::state::state_api::StateReader;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use conversions::felt::FromShortString;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;

#[test]
fn test_state_reverted() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Revert", &[]);

    // Mock contract just to get a class hash, it can be replaced with any other declared contract
    let mock_contract = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockCheckerLibCall",
        &[],
    );
    let mock_class_hash = cached_state.get_class_hash_at(mock_contract).unwrap();

    let res = call_contract_raw(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("call_contract_revert"),
        &[
            contract_address.into_(),
            selector_from_name("change_state_and_panic").into_(),
            mock_class_hash.into_(),
        ],
        TrackedResource::CairoSteps,
    )
    .unwrap();

    assert!(!res.execution.failed);
    let [inner_call] = &res.inner_calls[..] else {
        panic!("Expected one inner call, got {:?}", res.inner_calls);
    };
    assert_eq!(
        inner_call.execution.retdata.0,
        &[Felt::from_short_string("change_state_and_panic").unwrap()]
    );
    assert!(inner_call.execution.events.is_empty());
    assert!(inner_call.execution.l2_to_l1_messages.is_empty());
}

#[test]
fn test_tracked_resources() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_state = create_fork_cached_state_at(782_878, cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "TrackedResources",
        &[],
    );

    let main_call_info = call_contract_raw(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("call_twice"),
        &[],
        TrackedResource::SierraGas,
    )
    .unwrap();

    // `call_twice` from the `TrackedResources` contract
    assert!(!main_call_info.execution.failed);
    assert_eq!(main_call_info.inner_calls.len(), 2);
    assert_eq!(main_call_info.tracked_resource, TrackedResource::SierraGas);
    assert_ne!(main_call_info.execution.gas_consumed, 0);

    // `call_single` from the forked proxy contract
    let first_inner_call = main_call_info.inner_calls.first().unwrap();
    assert_eq!(
        first_inner_call.tracked_resource,
        TrackedResource::CairoSteps
    );
    assert_eq!(first_inner_call.execution.gas_consumed, 0);
    assert_ne!(first_inner_call.resources.n_steps, 0);
    assert_eq!(first_inner_call.inner_calls.len(), 1);

    // `call_internal` from the `TrackedResources` contract
    let inner_inner_call = first_inner_call.inner_calls.first().unwrap();
    assert_eq!(
        inner_inner_call.tracked_resource,
        TrackedResource::CairoSteps
    );
    assert_eq!(inner_inner_call.execution.gas_consumed, 0);
    assert_ne!(inner_inner_call.resources.n_steps, 0);

    // `call_internal` from the `TrackedResources` contract
    let second_inner_call = main_call_info.inner_calls.last().unwrap();
    assert_eq!(
        second_inner_call.tracked_resource,
        TrackedResource::SierraGas
    );
    assert_ne!(second_inner_call.execution.gas_consumed, 0);
    assert_eq!(second_inner_call.resources.n_steps, 0);
}
