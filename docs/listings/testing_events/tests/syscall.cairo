use snforge_std::{
    ContractClassTrait, DeclareResultTrait, Event, EventSpyAssertionsTrait, EventSpyTrait,
    EventsFilterTrait, declare, spy_events,
};
use testing_events::syscall::{
    ISpySyscallEventsCheckerDispatcher, ISpySyscallEventsCheckerDispatcherTrait,
};

#[test]
fn test_nonstandard_events() {
    let contract = declare("SpySyscallEventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ISpySyscallEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events();
    dispatcher.emit_event_with_syscall(123, 456);

    spy.assert_emitted(@array![(contract_address, Event { keys: array![123], data: array![456] })]);
}
