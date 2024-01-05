use crate::common::call_contract;
use crate::common::state::{create_cached_state, create_cheatnet_state};
use crate::common::{deploy_contract, felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cairo_lang_starknet::contract::starknet_keccak;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use conversions::felt252::FromShortString;
use conversions::IntoConv;
use std::vec;

fn felt_vec_to_event_vec(felts: &[Felt252]) -> Vec<Event> {
    let mut events = vec![];
    let mut i = 0;
    while i < felts.len() {
        let from = felts[i].clone().into_();
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length, _) = cheatnet_state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 1, "There should be one new event");

    let (length, _) = cheatnet_state.fetch_events(&Felt252::from(id));
    assert_eq!(length, 0, "There should be no new events");
}

#[test]
fn check_events_order() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let spy_events_checker_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );
    let spy_events_order_checker_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsOrderChecker",
        &[],
    );

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_and_call_another");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_order_checker_address,
        &selector,
        &[
            Felt252::from(123),
            Felt252::from(234),
            Felt252::from(345),
            spy_events_checker_address.into_(),
        ],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(234)]
        },
        "Wrong second event"
    );
    assert_eq!(
        events[2],
        Event {
            from: spy_events_order_checker_address,
            keys: vec![starknet_keccak("ThirdEvent".as_ref()).into()],
            data: vec![Felt252::from(345)]
        },
        "Wrong third event"
    );
}

#[test]
fn check_events_captured_only_for_spied_contracts() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let spy_events_checker_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );
    let selector = felt_selector_from_name("emit_one_event");

    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_checker_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let id = cheatnet_state.spy_events(SpyTarget::One(spy_events_checker_address));
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_checker_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt252::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn duplicate_spies_on_one_address() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );

    let id1 = cheatnet_state.spy_events(SpyTarget::One(contract_address));
    let id2 = cheatnet_state.spy_events(SpyTarget::One(contract_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length1, serialized_events1) = cheatnet_state.fetch_events(&Felt252::from(id1));
    let (length2, _) = cheatnet_state.fetch_events(&Felt252::from(id2));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("SpyEventsChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsLibCall",
        &[],
    );

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("call_lib_call");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123), class_hash.into_()],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ConstructorSpyEventsChecker",
        &[Felt252::from(123)],
    );

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("SpyEventsChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let spy_events_checker_address =
        deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
            .unwrap()
            .contract_address;
    let other_spy_events_checker_address =
        deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
            .unwrap()
            .contract_address;

    let id1 = cheatnet_state.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = cheatnet_state.spy_events(SpyTarget::One(other_spy_events_checker_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_checker_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length1, serialized_events1) = cheatnet_state.fetch_events(&Felt252::from(id1));
    let (length2, _) = cheatnet_state.fetch_events(&Felt252::from(id2));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let spy_events_checker_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("SpyEventsCheckerProxy").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let spy_events_checker_proxy_address = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[spy_events_checker_address.into_()],
    )
    .unwrap()
    .contract_address;
    let spy_events_checker_top_proxy_address = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[spy_events_checker_proxy_address.into_()],
    )
    .unwrap()
    .contract_address;

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_checker_top_proxy_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let spy_events_checker_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("SpyEventsCheckerProxy").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let spy_events_checker_proxy_address = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[spy_events_checker_address.into_()],
    )
    .unwrap()
    .contract_address;
    let spy_events_checker_top_proxy_address = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[spy_events_checker_proxy_address.into_()],
    )
    .unwrap()
    .contract_address;

    let id1 = cheatnet_state.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = cheatnet_state.spy_events(SpyTarget::One(spy_events_checker_proxy_address));
    let id3 = cheatnet_state.spy_events(SpyTarget::One(spy_events_checker_top_proxy_address));

    let selector = felt_selector_from_name("emit_one_event");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &spy_events_checker_top_proxy_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    let (length1, serialized_events1) = cheatnet_state.fetch_events(&Felt252::from(id1));
    let (length2, serialized_events2) = cheatnet_state.fetch_events(&Felt252::from(id2));
    let (length3, serialized_events3) = cheatnet_state.fetch_events(&Felt252::from(id3));
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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsChecker",
        &[],
    );

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("emit_event_syscall");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123), Felt252::from(456)],
    )
    .unwrap();

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));
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
