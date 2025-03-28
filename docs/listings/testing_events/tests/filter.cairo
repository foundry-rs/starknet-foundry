use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, spy_events, EventSpyAssertionsTrait,
    EventSpyTrait, Event,
    EventsFilterTrait, // Add for filtering the Events object (result of `get_events`)
};

use testing_events::contract::{
    SpyEventsChecker, ISpyEventsCheckerDispatcher, ISpyEventsCheckerDispatcherTrait
};

#[test]
fn test_assertions_with_filtering() {
    let contract = declare("SpyEventsChecker").unwrap().contract_class();
    let (first_address, _) = contract.deploy(@array![]).unwrap();
    let (second_address, _) = contract.deploy(@array![]).unwrap();

    let first_dispatcher = ISpyEventsCheckerDispatcher { contract_address: first_address };
    let second_dispatcher = ISpyEventsCheckerDispatcher { contract_address: second_address };

    let mut spy = spy_events();

    first_dispatcher.emit_one_event(123);
    second_dispatcher.emit_one_event(234);
    second_dispatcher.emit_one_event(345);

    let events_from_first_address = spy.get_events().emitted_by(first_address);
    let events_from_second_address = spy.get_events().emitted_by(second_address);

    let (from_first, event_from_first) = events_from_first_address.events.at(0);
    assert(from_first == @first_address, 'Emitted from wrong address');
    assert(event_from_first.data.at(0) == @123.into(), 'Data should be 123');

    let (from_second_one, event_from_second_one) = events_from_second_address.events.at(0);
    assert(from_second_one == @second_address, 'Emitted from wrong address');
    assert(event_from_second_one.data.at(0) == @234.into(), 'Data should be 234');

    let (from_second_two, event_from_second_two) = events_from_second_address.events.at(1);
    assert(from_second_two == @second_address, 'Emitted from wrong address');
    assert(event_from_second_two.data.at(0) == @345.into(), 'Data should be 345');
}
