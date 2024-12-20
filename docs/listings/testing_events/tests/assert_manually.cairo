use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, spy_events, EventSpyAssertionsTrait,
    EventSpyTrait, // Add for fetching events directly
    Event, // A structure describing a raw `Event`
};

use testing_events::contract::{
    SpyEventsChecker, ISpyEventsCheckerDispatcher, ISpyEventsCheckerDispatcherTrait
};

#[test]
fn test_complex_assertions() {
    let contract = declare("SpyEventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(); // Ad 1.

    dispatcher.emit_one_event(123);

    let events = spy.get_events(); // Ad 2.

    assert(events.events.len() == 1, 'There should be one event');

    let (from, event) = events.events.at(0); // Ad 3.
    assert(from == @contract_address, 'Emitted from wrong address');
    assert(event.keys.len() == 1, 'There should be one key');
    assert(event.keys.at(0) == @selector!("FirstEvent"), 'Wrong event name'); // Ad 4.
    assert(event.data.len() == 1, 'There should be one data');
}
