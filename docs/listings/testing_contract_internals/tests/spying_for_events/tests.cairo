use core::array::ArrayTrait;
use snforge_std::{
    declare, ContractClassTrait, spy_events, EventSpy, EventSpyTrait, EventSpyAssertionsTrait,
    Event, test_address
};
use testing_contract_internals::spying_for_events::Emitter;

#[test]
fn test_expect_event() {
    let contract_address = test_address();
    let mut spy = spy_events();

    let mut testing_state = Emitter::contract_state_for_testing();
    Emitter::emit_event(ref testing_state);

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Emitter::Event::ThingEmitted(Emitter::ThingEmitted { thing: 420 })
                )
            ]
        )
}
