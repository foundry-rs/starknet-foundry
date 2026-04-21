use debugging_events::{IEventsCheckerDispatcher, IEventsCheckerDispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare};

#[test]
fn test_debugging_trace_events_component() {
    let contract_class = declare("EventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract_class.deploy(@array![]).unwrap();

    IEventsCheckerDispatcher { contract_address }.emit_event(42);
}

#[test]
fn test_debugging_trace_eventless_success() {
    let contract_class = declare("EventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract_class.deploy(@array![]).unwrap();

    IEventsCheckerDispatcher { contract_address }.do_not_emit();
}
