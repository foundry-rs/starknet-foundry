use crate::common::state::create_cheatnet_state;
use crate::common::{deploy_contract, felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use cheatnet::cheatcodes::spy_events::{Event, SpyTarget};
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;
use std::vec;

fn felt_vec_to_event_vec(felts: &[Felt252]) -> Vec<Event> {
    let mut events = vec![];
    let mut i = 0;
    while i < felts.len() {
        let from = felts[i].to_contract_address();
        let keys_length = &felts[i + 1];
        let keys = &felts[i + 2..i + 2 + felt_to_usize(keys_length).unwrap()];
        let data_length = &felts[i + 2 + felt_to_usize(keys_length).unwrap()];
        let data = &felts[i + 2 + felt_to_usize(keys_length).unwrap() + 1
            ..i + 2
                + felt_to_usize(keys_length).unwrap()
                + 1
                + felt_to_usize(data_length).unwrap()];

        events.push(Event {
            from,
            keys: Vec::from(keys),
            data: Vec::from(data),
        });

        i = i + 2 + felt_to_usize(keys_length).unwrap() + 1 + felt_to_usize(data_length).unwrap();
    }

    events
}

#[test]
fn spy_events_complex() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 1, "There should be one event");
    assert_eq!(
        events.len(),
        length,
        "Length after serialization should be the same"
    );
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length, _) = state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 1, "There should be one new event");

    let (length, _) = state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 0, "There should be no new events");
}

#[test]
fn check_events_order() {
    let mut state = create_cheatnet_state();

    let spy_events_checker_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);
    let spy_events_order_checker_address =
        deploy_contract(&mut state, "SpyEventsOrderChecker", &[]);

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_and_call_another");
    call_contract(
        &spy_events_order_checker_address,
        &selector,
        &[
            Felt252::from(123),
            Felt252::from(234),
            Felt252::from(345),
            spy_events_checker_address.to_felt252(),
        ],
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 3, "There should be three events");
    assert_eq!(
        events[0],
        Event {
            from: spy_events_order_checker_address,
            keys: vec![starknet_keccak("SecondEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong first event"
    );
    assert_eq!(
        events[1],
        Event {
            from: spy_events_order_checker_address,
            keys: vec![starknet_keccak("ThirdEvent".as_ref()).into()],
            data: vec![Felt252::from(345)]
        },
        "Wrong second event"
    );
    assert_eq!(
        events[2],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(234)]
        },
        "Wrong third event"
    );
}

#[test]
fn duplicate_spies_on_one_address() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);

    let id1 = state.spy_events(SpyTarget::One(contract_address));
    let id2 = state.spy_events(SpyTarget::One(contract_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length1, serialized_events1) = state.fetch_events(&Felt252::from(id1));
    let (length2, _) = state.fetch_events(&Felt252::from(id2));
    let events1 = felt_vec_to_event_vec(&serialized_events1);

    assert_eq!(length1, 1, "There should be one event");
    assert_eq!(length2, 0, "There should be no events");
    assert_eq!(
        events1[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn library_call_emits_event() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "SpyEventsChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = deploy_contract(&mut state, "SpyEventsLibCall", &[]);

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("call_lib_call");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123), class_hash.to_felt252()],
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn event_emitted_in_constructor() {
    let mut state = create_cheatnet_state();

    let id = state.spy_events(SpyTarget::All);

    let contract_address = deploy_contract(
        &mut state,
        "ConstructorSpyEventsChecker",
        &[Felt252::from(123)],
    );

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 1, "There should be one event");
    assert_eq!(
        events.len(),
        length,
        "Length after serialization should be the same"
    );
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn check_if_there_is_no_interference() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = "SpyEventsChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let spy_events_checker_address = state.deploy(&class_hash, &[]).unwrap().contract_address;
    let other_spy_events_checker_address = state.deploy(&class_hash, &[]).unwrap().contract_address;

    let id1 = state.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = state.spy_events(SpyTarget::One(other_spy_events_checker_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &spy_events_checker_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length1, serialized_events1) = state.fetch_events(&Felt252::from(id1));
    let (length2, _) = state.fetch_events(&Felt252::from(id2));
    let events1 = felt_vec_to_event_vec(&serialized_events1);

    assert_eq!(length1, 1, "There should be one event");
    assert_eq!(length2, 0, "There should be no events");
    assert_eq!(
        events1[0],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn test_nested_calls() {
    let mut state = create_cheatnet_state();

    let spy_events_checker_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);

    let contracts = get_contracts();

    let contract_name = "SpyEventsCheckerProxy".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let spy_events_checker_proxy_address = state
        .deploy(&class_hash, &[spy_events_checker_address.to_felt252()])
        .unwrap()
        .contract_address;
    let spy_events_checker_top_proxy_address = state
        .deploy(
            &class_hash,
            &[spy_events_checker_proxy_address.to_felt252()],
        )
        .unwrap()
        .contract_address;

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &spy_events_checker_top_proxy_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 3, "There should be three events");
    assert_eq!(
        events[0],
        Event {
            from: spy_events_checker_top_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong first event"
    );
    assert_eq!(
        events[1],
        Event {
            from: spy_events_checker_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong second event"
    );
    assert_eq!(
        events[2],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong third event"
    );
}

#[test]
fn use_multiple_spies() {
    let mut state = create_cheatnet_state();

    let spy_events_checker_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);

    let contracts = get_contracts();

    let contract_name = "SpyEventsCheckerProxy".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let spy_events_checker_proxy_address = state
        .deploy(&class_hash, &[spy_events_checker_address.to_felt252()])
        .unwrap()
        .contract_address;
    let spy_events_checker_top_proxy_address = state
        .deploy(
            &class_hash,
            &[spy_events_checker_proxy_address.to_felt252()],
        )
        .unwrap()
        .contract_address;

    let id1 = state.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = state.spy_events(SpyTarget::One(spy_events_checker_proxy_address));
    let id3 = state.spy_events(SpyTarget::One(spy_events_checker_top_proxy_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &spy_events_checker_top_proxy_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    let (length1, serialized_events1) = state.fetch_events(&Felt252::from(id1));
    let (length2, serialized_events2) = state.fetch_events(&Felt252::from(id2));
    let (length3, serialized_events3) = state.fetch_events(&Felt252::from(id3));
    let events1 = felt_vec_to_event_vec(&serialized_events1);
    let events2 = felt_vec_to_event_vec(&serialized_events2);
    let events3 = felt_vec_to_event_vec(&serialized_events3);

    assert_eq!(length1, 1, "There should be one event");
    assert_eq!(length2, 1, "There should be one event");
    assert_eq!(length3, 1, "There should be one event");

    assert_eq!(
        events1[0],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong spy_events_checker event"
    );
    assert_eq!(
        events2[0],
        Event {
            from: spy_events_checker_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong spy_events_checker_proxy event"
    );
    assert_eq!(
        events3[0],
        Event {
            from: spy_events_checker_top_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong spy_events_checker_top_proxy event"
    );
}

#[test]
fn test_emitted_by_emit_events_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpyEventsChecker", &[]);

    let id = state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_event_syscall");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123), Felt252::from(456)],
        &mut state,
    )
    .unwrap();

    let (length, serialized_events) = state.fetch_events(&Felt252::from(id));
    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![Felt252::from(123)],
            data: vec![Felt252::from(456)]
        },
        "Wrong spy_events_checker event"
    );
}
