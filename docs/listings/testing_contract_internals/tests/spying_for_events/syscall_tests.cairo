use core::array::ArrayTrait;
use core::result::ResultTrait;
use starknet::{ContractAddress, SyscallResultTrait, syscalls::emit_event_syscall};
use snforge_std::{
    declare, ContractClassTrait, spy_events, EventSpy, EventSpyTrait, EventSpyAssertionsTrait,
    Event, test_address,
};

#[test]
fn test_expect_event() {
    let contract_address = test_address();
    let mut spy = spy_events();

    emit_event_syscall(array![1234].span(), array![2345].span()).unwrap_syscall();

    spy
        .assert_emitted(
            @array![(contract_address, Event { keys: array![1234], data: array![2345] })],
        );

    assert(spy.get_events().events.len() == 1, 'There should no more events');
}
