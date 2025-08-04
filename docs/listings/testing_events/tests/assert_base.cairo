use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, spy_events,
    EventSpyAssertionsTrait, // Add for assertions on the EventSpy
    Event // Import the base Event
};

use testing_events::contract::{
    SpyEventsChecker, ISpyEventsCheckerDispatcher, ISpyEventsCheckerDispatcherTrait,
};

#[test]
fn test_simple_assertions() {
    let contract = declare("SpyEventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events();

    dispatcher.emit_one_event(123);

    let mut keys = array![];
    keys.append(selector!("FirstEvent")); // Append the name of the event to keys
    let mut data = array![];
    data.append(123); // Append the expected data

    let expected = Event { keys, data }; // Instantiate the Event

    spy.assert_emitted(@array![ // Assert
    (contract_address, expected)]);
}
