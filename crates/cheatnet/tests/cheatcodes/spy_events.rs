use crate::cheatcodes::test_environment::TestEnvironment;
use crate::common::get_contracts;
use crate::common::state::create_fork_cached_state_at;
use crate::common::{call_contract, deploy_contract, felt_selector_from_name};
use cairo_lang_starknet_classes::keccak::starknet_keccak;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event;
use cheatnet::state::CheatnetState;
use conversions::string::TryFromHexStr;
use conversions::IntoConv;
use starknet_types_core::felt::Felt;
use std::vec;
use tempfile::TempDir;

trait SpyTrait {
    fn get_events(&mut self, id: usize) -> Vec<Event>;
}

impl SpyTrait for TestEnvironment {
    fn get_events(&mut self, events_offset: usize) -> Vec<Event> {
        self.cheatnet_state.get_events(events_offset)
    }
}

#[test]
fn spy_events_zero_offset() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong event"
    );

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);

    let length = test_env.get_events(0).len();
    assert_eq!(length, 2, "There should be one more new event");
}

#[test]
fn spy_events_some_offset() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);
    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);
    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);

    let events = test_env.get_events(2);

    assert_eq!(
        events.len(),
        1,
        "There should be only one event fetched after accounting for an offset of 2"
    );
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong event"
    );

    test_env.call_contract(&contract_address, "emit_one_event", &[Felt::from(123)]);

    let length = test_env.get_events(2).len();
    assert_eq!(length, 2, "There should be one more new event");
}

#[test]
fn check_events_order() {
    let mut test_env = TestEnvironment::new();

    let spy_events_checker_address = test_env.deploy("SpyEventsChecker", &[]);
    let spy_events_order_checker_address = test_env.deploy("SpyEventsOrderChecker", &[]);

    test_env.call_contract(
        &spy_events_order_checker_address,
        "emit_and_call_another",
        &[
            Felt::from(123),
            Felt::from(234),
            Felt::from(345),
            spy_events_checker_address.into_(),
        ],
    );

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 3, "There should be three events");
    assert_eq!(
        events[0],
        Event {
            from: spy_events_order_checker_address,
            keys: vec![starknet_keccak("SecondEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong first event"
    );
    assert_eq!(
        events[1],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(234)]
        },
        "Wrong second event"
    );
    assert_eq!(
        events[2],
        Event {
            from: spy_events_order_checker_address,
            keys: vec![starknet_keccak("ThirdEvent".as_ref()).into()],
            data: vec![Felt::from(345)]
        },
        "Wrong third event"
    );
}

#[test]
fn library_call_emits_event() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("SpyEventsChecker", &contracts_data);
    let contract_address = test_env.deploy("SpyEventsLibCall", &[]);

    test_env.call_contract(
        &contract_address,
        "call_lib_call",
        &[Felt::from(123), class_hash.into_()],
    );

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong event"
    );
}

#[test]
fn event_emitted_in_constructor() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ConstructorSpyEventsChecker", &[Felt::from(123)]);

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
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

    test_env.call_contract(
        &spy_events_checker_top_proxy_address,
        "emit_one_event",
        &[Felt::from(123)],
    );

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 3, "There should be three events");
    assert_eq!(
        events[0],
        Event {
            from: spy_events_checker_top_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong first event"
    );
    assert_eq!(
        events[1],
        Event {
            from: spy_events_checker_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong second event"
    );
    assert_eq!(
        events[2],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
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

    // Events emitted in the order of:
    // - spy_events_checker_top_proxy_address,
    // - spy_events_checker_proxy_address,
    // - spy_events_checker_address
    test_env.call_contract(
        &spy_events_checker_top_proxy_address,
        "emit_one_event",
        &[Felt::from(123)],
    );

    let events1 = test_env.get_events(0);
    let events2 = test_env.get_events(1);
    let events3 = test_env.get_events(2);

    assert_eq!(events1.len(), 3, "There should be one event");
    assert_eq!(events2.len(), 2, "There should be one event");
    assert_eq!(events3.len(), 1, "There should be one event");

    assert_eq!(
        events1[0],
        Event {
            from: spy_events_checker_top_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong spy_events_checker_top_proxy event"
    );
    assert_eq!(
        events2[0],
        Event {
            from: spy_events_checker_proxy_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong spy_events_checker_proxy event"
    );
    assert_eq!(
        events3[0],
        Event {
            from: spy_events_checker_address,
            keys: vec![starknet_keccak("FirstEvent".as_ref()).into()],
            data: vec![Felt::from(123)]
        },
        "Wrong spy_events_checker event"
    );
}

#[test]
fn test_emitted_by_emit_events_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("SpyEventsChecker", &[]);

    test_env.call_contract(
        &contract_address,
        "emit_event_syscall",
        &[Felt::from(123), Felt::from(456)],
    );

    let events = test_env.get_events(0);

    assert_eq!(events.len(), 1, "There should be one event");
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![Felt::from(123)],
            data: vec![Felt::from(456)]
        },
        "Wrong spy_events_checker event"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn capture_cairo0_event() {
    let temp_dir = TempDir::new().unwrap();
    let mut cached_state = create_fork_cached_state_at(53_626, temp_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "SpyEventsCairo0",
        &[],
    );

    let selector = felt_selector_from_name("test_cairo0_event_collection");

    let cairo0_contract_address =
        Felt::try_from_hex_str("0x2c77ca97586968c6651a533bd5f58042c368b14cf5f526d2f42f670012e10ac")
            .unwrap();

    call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[cairo0_contract_address],
    );

    let events = cheatnet_state.get_events(0);

    assert_eq!(events.len(), 1, "There should be one event");

    assert_eq!(
        events[0],
        Event {
            from: cairo0_contract_address.into_(),
            keys: vec![starknet_keccak("my_event".as_ref()).into()],
            data: vec![Felt::from(123_456_789)]
        },
        "Wrong spy_events_checker event"
    );
}
