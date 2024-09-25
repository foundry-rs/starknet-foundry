use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, spy_events,
    EventSpyAssertionsTrait, // Add for assertions on the EventSpy
};

use testing_events::contract::{
    SpyEventsChecker, ISpyEventsCheckerDispatcher, ISpyEventsCheckerDispatcherTrait
};

#[test]
fn test_simple_assertions() {
    let contract = declare("SpyEventsChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(); // Ad. 1

    dispatcher.emit_one_event(123);

    spy
        .assert_emitted(
            @array![ // Ad. 2
                (
                    contract_address,
                    SpyEventsChecker::Event::FirstEvent(
                        SpyEventsChecker::FirstEvent { some_data: 123 }
                    )
                )
            ]
        );
}
