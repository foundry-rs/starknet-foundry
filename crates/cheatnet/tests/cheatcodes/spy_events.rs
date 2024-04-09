use crate::cheatcodes::test_environment::TestEnvironment;
use crate::common::get_contracts;
use crate::common::state::create_fork_cached_state_at;
use crate::common::{call_contract, deploy_contract, felt_selector_from_name};
use cairo_felt::Felt252;
use cairo_lang_starknet_classes::keccak::starknet_keccak;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use cheatnet::state::CheatnetState;
use conversions::string::TryFromHexStr;
use conversions::IntoConv;
use std::vec;
use tempfile::TempDir;

trait SpyTrait {
    fn spy_events(&mut self, spy_on: SpyTarget) -> usize;
    fn fetch_events(&mut self, id: usize) -> (usize, Vec<Felt252>);
}

impl SpyTrait for TestEnvironment {
    fn spy_events(&mut self, spy_on: SpyTarget) -> usize {
        self.cheatnet_state.spy_events(spy_on)
    }

    fn fetch_events(&mut self, id: usize) -> (usize, Vec<Felt252>) {
        self.cheatnet_state.fetch_events(&Felt252::from(id))
    }
}

pub fn felt_vec_to_event_vec(felts: &[Felt252]) -> Vec<Event> {
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
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    let id = test_env.spy_events(SpyTarget::All);

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt252::from(123)]);

    let (length, serialized_events) = test_env.fetch_events(id);
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

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt252::from(123)]);

    let (length, _) = test_env.fetch_events(id);
    assert_eq!(length, 1, "There should be one new event");

    let (length, _) = test_env.fetch_events(id);
    assert_eq!(length, 0, "There should be no new events");
}

#[test]
fn check_events_order() {
    let mut test_env = TestEnvironment::new();

    let spy_events_checker_address = test_env.deploy("SpyEventsChecker", &[]);
    let spy_events_order_checker_address = test_env.deploy("SpyEventsOrderChecker", &[]);

    let id = test_env.spy_events(SpyTarget::All);

    test_env.call_contract(
        &spy_events_order_checker_address,
        "emit_and_call_another",
        &[
            Felt252::from(123),
            Felt252::from(234),
            Felt252::from(345),
            spy_events_checker_address.into_(),
        ],
    );

    let (length, serialized_events) = test_env.fetch_events(id);
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
    let mut test_env = TestEnvironment::new();

    let spy_events_checker_address = test_env.deploy("SpyEventsChecker", &[]);

    test_env.call_contract(
        &spy_events_checker_address,
        "emit_one_event",
        &[Felt252::from(123)],
    );

    let id = test_env.spy_events(SpyTarget::One(spy_events_checker_address));
    test_env.call_contract(
        &spy_events_checker_address,
        "emit_one_event",
        &[Felt252::from(123)],
    );

    let (length, serialized_events) = test_env.fetch_events(id);
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
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    let id1 = test_env.spy_events(SpyTarget::One(contract_address));
    let id2 = test_env.spy_events(SpyTarget::One(contract_address));

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt252::from(123)]);

    let (length1, serialized_events1) = test_env.fetch_events(id1);
    let (length2, _) = test_env.fetch_events(id2);
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("SpyEventsChecker", &contracts_data);
    let contract_address = test_env.deploy("SpyEventsLibCall", &[]);

    let id = test_env.spy_events(SpyTarget::All);

    test_env.call_contract(
        &contract_address,
        "call_lib_call",
        &[Felt252::from(123), class_hash.into_()],
    );

    let (length, serialized_events) = test_env.fetch_events(id);
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
    let mut test_env = TestEnvironment::new();

    let id = test_env.spy_events(SpyTarget::All);

    let contract_address = test_env.deploy("ConstructorSpyEventsChecker", &[Felt252::from(123)]);

    let (length, serialized_events) = test_env.fetch_events(id);
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("SpyEventsChecker", &contracts_data);

    let spy_events_checker_address = test_env.deploy_wrapper(&class_hash, &[]);
    let other_spy_events_checker_address = test_env.deploy_wrapper(&class_hash, &[]);

    let id1 = test_env.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = test_env.spy_events(SpyTarget::One(other_spy_events_checker_address));

    test_env.call_contract(
        &spy_events_checker_address,
        "emit_one_event",
        &[Felt252::from(123)],
    );

    let (length1, serialized_events1) = test_env.fetch_events(id1);
    let (length2, _) = test_env.fetch_events(id2);
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
    let mut test_env = TestEnvironment::new();

    let spy_events_checker_address = test_env.deploy("SpyEventsChecker", &[]);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("SpyEventsCheckerProxy", &contracts_data);

    let spy_events_checker_proxy_address =
        test_env.deploy_wrapper(&class_hash, &[spy_events_checker_address.into_()]);
    let spy_events_checker_top_proxy_address =
        test_env.deploy_wrapper(&class_hash, &[spy_events_checker_proxy_address.into_()]);

    let id = test_env.spy_events(SpyTarget::All);

    test_env.call_contract(
        &spy_events_checker_top_proxy_address,
        "emit_one_event",
        &[Felt252::from(123)],
    );

    let (length, serialized_events) = test_env.fetch_events(id);
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
    let mut test_env = TestEnvironment::new();

    let spy_events_checker_address = test_env.deploy("SpyEventsChecker", &[]);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("SpyEventsCheckerProxy", &contracts_data);

    let spy_events_checker_proxy_address =
        test_env.deploy_wrapper(&class_hash, &[spy_events_checker_address.into_()]);
    let spy_events_checker_top_proxy_address =
        test_env.deploy_wrapper(&class_hash, &[spy_events_checker_proxy_address.into_()]);

    let id1 = test_env.spy_events(SpyTarget::One(spy_events_checker_address));
    let id2 = test_env.spy_events(SpyTarget::One(spy_events_checker_proxy_address));
    let id3 = test_env.spy_events(SpyTarget::One(spy_events_checker_top_proxy_address));

    test_env.call_contract(
        &spy_events_checker_top_proxy_address,
        "emit_one_event",
        &[Felt252::from(123)],
    );

    let (length1, serialized_events1) = test_env.fetch_events(id1);
    let (length2, serialized_events2) = test_env.fetch_events(id2);
    let (length3, serialized_events3) = test_env.fetch_events(id3);
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
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    let id = test_env.spy_events(SpyTarget::All);

    test_env.call_contract(
        &contract_address,
        "emit_event_syscall",
        &[Felt252::from(123), Felt252::from(456)],
    );

    let (length, serialized_events) = test_env.fetch_events(id);
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

#[test]
fn capture_cairo0_event() {
    let temp_dir = TempDir::new().unwrap();
    let mut cached_state = create_fork_cached_state_at(53_626, temp_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "SpyEventsCairo0",
        &[],
    );

    let id = cheatnet_state.spy_events(SpyTarget::All);

    let selector = felt_selector_from_name("test_cairo0_event_collection");

    let cairo0_contract_address = Felt252::try_from_hex_str(
        "0x2c77ca97586968c6651a533bd5f58042c368b14cf5f526d2f42f670012e10ac",
    )
    .unwrap();

    call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[cairo0_contract_address.clone()],
    );

    let (length, serialized_events) = cheatnet_state.fetch_events(&Felt252::from(id));

    let events = felt_vec_to_event_vec(&serialized_events);

    assert_eq!(length, 1, "There should be one event");

    assert_eq!(
        events[0],
        Event {
            from: cairo0_contract_address.into_(),
            keys: vec![starknet_keccak("my_event".as_ref()).into()],
            data: vec![Felt252::from(123_456_789)]
        },
        "Wrong spy_events_checker event"
    );
}
