use crate::common::state::create_cached_state;
use crate::common::{call_contract_raw, deploy_contract, felt_selector_from_name};
use blockifier::state::state_api::StateReader;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use conversions::felt::FromShortString;
use starknet_types_core::felt::Felt;

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
        felt_selector_from_name("call_contract_revert"),
        &[
            contract_address.into_(),
            felt_selector_from_name("change_state_and_panic").into_(),
            mock_class_hash.into_(),
        ],
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
