use crate::common::state::{create_cached_state, create_fork_cached_state_at};
use crate::common::{
    call_contract_extended_result, deploy_contract, execute_entry_point_without_revert,
    selector_from_name,
};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::syscalls::hint_processor::ENTRYPOINT_FAILED_ERROR_FELT;
use blockifier::state::state_api::StateReader;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallFailure;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use conversions::felt::FromShortString;
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

    let res = execute_entry_point_without_revert(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_in_nested_call_and_handle_panic"),
        &[contract_address.into_(), mock_class_hash.into_()],
        TrackedResource::SierraGas,
    )
    .unwrap();

    assert!(!res.execution.failed);
    let [inner_call] = &res.inner_calls[..] else {
        panic!("Expected one inner call, got {:?}", res.inner_calls);
    };
    assert_eq!(
        inner_call.execution.retdata.0,
        &[Felt::from_short_string("modify_contract_var_and_panic").unwrap()]
    );
    assert!(inner_call.execution.events.is_empty());
    assert!(inner_call.execution.l2_to_l1_messages.is_empty());
}

#[test]
fn test_state_not_reverted_in_top_call_when_raw_execution() {
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

    // Call via `execute_call_entry_point` directly (no revert on failure) to confirm
    // that state mutations (events, messages) survive a failed call at this layer.
    let res = execute_entry_point_without_revert(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_contract_var_and_panic"),
        &[mock_class_hash.into_()],
        TrackedResource::SierraGas,
    )
    .unwrap();

    assert!(res.execution.failed);
    assert!(res.inner_calls.is_empty());
    assert!(!res.execution.events.is_empty());
    assert!(!res.execution.l2_to_l1_messages.is_empty());
}

#[test]
fn test_state_reverted_in_top_call_when_call_entry_point() {
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

    // Call via `call_entry_point`, so the same way contract calls are executed in test bodies.
    // On failure, it should revert all state mutations.
    let res = call_contract_extended_result(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_contract_var_and_panic"),
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
        &[Felt::from_short_string("modify_contract_var_and_panic").unwrap()]
    );
}

#[test]
fn test_state_reverted_only_in_failed_nested_call_when_raw_execution() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Revert", &[]);
    let storage_key = StorageKey::try_from(felt!(123_u64)).unwrap();

    // Call via `execute_call_entry_point` directly (no revert on failure) to confirm
    // that state mutations (storage, events, messages) survive a failed call at this layer.
    let res = execute_entry_point_without_revert(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_in_top_and_nested_calls_and_panic"),
        &[(*storage_key.0).into_()],
        TrackedResource::SierraGas,
    )
    .unwrap();

    assert!(res.execution.failed);
    let [inner_call, failed_inner_call] = &res.inner_calls[..] else {
        panic!("Expected two inner calls, got {:?}", res.inner_calls);
    };
    // Successful inner call state was not reverted.
    assert!(!inner_call.execution.failed);
    assert!(!inner_call.execution.events.is_empty());
    assert!(!inner_call.execution.l2_to_l1_messages.is_empty());

    // Failed inner call state was reverted by `execute_inner_call` function.
    assert!(failed_inner_call.execution.failed);
    assert!(failed_inner_call.execution.events.is_empty());
    assert!(failed_inner_call.execution.l2_to_l1_messages.is_empty());

    assert!(!res.execution.events.is_empty());
    assert!(!res.execution.l2_to_l1_messages.is_empty());
    assert_eq!(
        res.execution.retdata.0,
        &[
            Felt::from_short_string("modify_specific_storage").unwrap(),
            ENTRYPOINT_FAILED_ERROR_FELT
        ]
    );

    let storage_value = cached_state
        .get_storage_at(contract_address, storage_key)
        .unwrap();
    assert_eq!(
        storage_value,
        felt!(99_u8),
        "Storage should be 99 without revert"
    );
}

#[test]
fn test_state_reverted_in_top_and_nested_calls_when_call_entry_point() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Revert", &[]);
    let storage_key = StorageKey::try_from(felt!(123_u64)).unwrap();

    // Call via `call_entry_point`, so the same way contract calls are executed in test bodies.
    // On failure, it should revert all state mutations.
    let res = call_contract_extended_result(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("modify_in_top_and_nested_calls_and_panic"),
        &[(*storage_key.0).into_()],
    );

    let res_call_info = res.call_info.expect("Call info should be present");
    assert!(res_call_info.execution.failed);
    let [inner_call, failed_inner_call] = &res_call_info.inner_calls[..] else {
        panic!(
            "Expected two inner calls, got {:?}",
            res_call_info.inner_calls
        );
    };
    // Now state in all nested calls should be reverted.
    assert!(!inner_call.execution.failed);
    assert!(inner_call.execution.events.is_empty());
    assert!(inner_call.execution.l2_to_l1_messages.is_empty());
    assert!(failed_inner_call.execution.failed);
    assert!(failed_inner_call.execution.events.is_empty());
    assert!(failed_inner_call.execution.l2_to_l1_messages.is_empty());

    assert!(res_call_info.execution.events.is_empty());
    assert!(res_call_info.execution.l2_to_l1_messages.is_empty());
    assert_eq!(
        res_call_info.execution.retdata.0,
        &[
            Felt::from_short_string("modify_specific_storage").unwrap(),
            ENTRYPOINT_FAILED_ERROR_FELT
        ]
    );

    let CallFailure::Recoverable { panic_data } = res.call_result.as_ref().unwrap_err() else {
        panic!("Expected Recoverable error, got {:?}", res.call_result);
    };
    assert_eq!(
        panic_data,
        &[
            Felt::from_short_string("modify_specific_storage").unwrap(),
            ENTRYPOINT_FAILED_ERROR_FELT,
            ENTRYPOINT_FAILED_ERROR_FELT
        ]
    );

    let storage_value = cached_state
        .get_storage_at(contract_address, storage_key)
        .unwrap();
    assert_eq!(
        storage_value,
        felt!(0_u8),
        "Storage should be 0 after revert"
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

    let main_call_info = execute_entry_point_without_revert(
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
