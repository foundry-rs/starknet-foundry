use crate::common::state::{create_cached_state, create_fork_cached_state_at};
use crate::common::{
    call_contract_extended_result, call_contract_raw, deploy_contract, selector_from_name,
};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::syscalls::hint_processor::ENTRYPOINT_FAILED_ERROR_FELT;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use conversions::IntoConv;
use conversions::felt::FromShortString;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::felt;
use starknet_api::state::StorageKey;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;

#[test]
fn test_state_reverted_in_nested_call() {
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
        selector_from_name("modify_in_nested_call_and_handle_panic"),
        &[
            contract_address.into_(),
            selector_from_name("modify_state_and_panic").into_(),
            mock_class_hash.into_(),
        ],
        TrackedResource::SierraGas,
    )
    .unwrap();

    assert!(!res.execution.failed);
    let [inner_call] = &res.inner_calls[..] else {
        panic!("Expected one inner call, got {:?}", res.inner_calls);
    };
    assert_eq!(
        inner_call.execution.retdata.0,
        &[Felt::from_short_string("modify_state_and_panic").unwrap()]
    );
    assert!(inner_call.execution.events.is_empty());
    assert!(inner_call.execution.l2_to_l1_messages.is_empty());
}

#[test]
fn test_state_reverted_in_top_call() {
    let reset_env = || -> (CachedState<ExtendedStateReader>, CheatnetState, ContractAddress, ClassHash) {
        let mut cached_state = create_cached_state();
        let mut cheatnet_state = CheatnetState::default();

        // Mock contract just to get a class hash, it can be replaced with any other declared contract
        let mock_contract = deploy_contract(
            &mut cached_state,
            &mut cheatnet_state,
            "MockCheckerLibCall",
            &[],
        );
        let mock_class_hash = cached_state.get_class_hash_at(mock_contract).unwrap();
        let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Revert", &[]);

        (cached_state, cheatnet_state, contract_address, mock_class_hash)
    };
    let (mut cached_state, mut cheatnet_state, contract_address, mock_class_hash) = reset_env();

    // First call entrypoint directly to make sure that the state is then modified but not reverted.
    let res_direct_call = call_contract_raw(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_state_and_panic"),
        &[mock_class_hash.into_()],
        TrackedResource::SierraGas,
    )
    .unwrap();
    assert!(res_direct_call.execution.failed);
    assert!(res_direct_call.inner_calls.is_empty());
    assert!(!res_direct_call.execution.events.is_empty());
    assert!(!res_direct_call.execution.l2_to_l1_messages.is_empty());

    // Now call the same entrypoint through the `call_entry_point` function, hence
    // the same way as contract calls are executed within the test body.
    let (mut cached_state, mut cheatnet_state, contract_address, mock_class_hash) = reset_env();
    let res = call_contract_extended_result(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_state_and_panic"),
        &[mock_class_hash.into_()],
    )
    .call_info
    .expect("Call info should be present");

    assert!(res.execution.failed);
    assert!(res.inner_calls.is_empty());
    assert!(res.execution.events.is_empty());
    assert!(res.execution.l2_to_l1_messages.is_empty());
    assert_eq!(
        res.execution.retdata.0,
        &[Felt::from_short_string("modify_state_and_panic").unwrap()]
    );
}

#[test]
fn test_state_reverted_top_and_nested_calls() {
    let reset_env = || -> (CachedState<ExtendedStateReader>, CheatnetState, ContractAddress, StorageKey) {
        let mut cached_state = create_cached_state();
        let mut cheatnet_state = CheatnetState::default();

        let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Revert", &[]);
        let storage_key = StorageKey::try_from(felt!(123_u64)).unwrap();
        (cached_state, cheatnet_state, contract_address, storage_key)
    };
    let (mut cached_state, mut cheatnet_state, contract_address, storage_key) = reset_env();

    // First call entrypoint directly to make sure that the state is then modified but not reverted.
    let res_direct_call = call_contract_raw(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_in_top_and_nested_calls_and_panic"),
        &[(*storage_key.0).into_()],
        TrackedResource::SierraGas,
    )
    .unwrap();

    assert!(res_direct_call.execution.failed);
    assert!(!res_direct_call.inner_calls.is_empty());
    assert!(!res_direct_call.execution.events.is_empty());
    assert!(!res_direct_call.execution.l2_to_l1_messages.is_empty());

    let storage_value = cached_state
        .get_storage_at(contract_address, storage_key)
        .unwrap();
    assert_eq!(
        storage_value,
        felt!(99_u8),
        "Storage should be 99 without revert"
    );

    // Now call the same entrypoint through the `call_entry_point` function, hence
    // the same way as contract calls are executed within the test body.
    let (mut cached_state, mut cheatnet_state, contract_address, storage_key) = reset_env();
    let res = call_contract_extended_result(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_in_top_and_nested_calls_and_panic"),
        &[(*storage_key.0).into_()],
    )
    .call_info
    .expect("Call info should be present");

    let storage_value = cached_state
        .get_storage_at(contract_address, storage_key)
        .unwrap();
    assert_eq!(
        storage_value,
        felt!(0_u8),
        "Storage should be 0 after revert"
    );

    assert!(res.execution.failed);
    assert!(!res.inner_calls.is_empty());
    assert!(res.execution.events.is_empty());
    assert!(res.execution.l2_to_l1_messages.is_empty());
    assert_eq!(
        res.execution.retdata.0,
        &[
            Felt::from_short_string("modify_specific_storage").unwrap(),
            ENTRYPOINT_FAILED_ERROR_FELT
        ]
    );
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
