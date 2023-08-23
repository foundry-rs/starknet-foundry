use crate::common::deploy_contract;
use crate::common::state::create_cheatnet_state;
use cairo_felt::Felt252;
use cairo_lang_starknet::contract::starknet_keccak;
use cheatnet::cheatcodes::spy_events::SpyTarget;
use cheatnet::conversions::{contract_address_to_felt, felt_selector_from_name};
use cheatnet::rpc::call_contract;

#[test]
fn spy_events_complex() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpyEventsChecker", vec![].as_slice());

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &contract_address,
        &selector,
        vec![Felt252::from(123)].as_slice(),
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));

    assert_eq!(length, 1, "There should be one event");
    assert_eq!(
        serialized_events[0],
        contract_address_to_felt(contract_address),
        "Wrong emitted"
    );
    assert_eq!(
        serialized_events[1],
        starknet_keccak("FirstEvent".as_ref()).into(),
        "Wrong event name"
    );
    assert_eq!(
        serialized_events[2],
        Felt252::from(0),
        "There should be no keys"
    );
    assert_eq!(
        serialized_events[3],
        Felt252::from(1),
        "There should be one data"
    );

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &contract_address,
        &selector,
        vec![Felt252::from(123)].as_slice(),
        &mut state,
    )
    .unwrap();

    let (length, _) = state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 1, "There should be one new event");

    let (length, _) = state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 0, "There should be no new events");
}
