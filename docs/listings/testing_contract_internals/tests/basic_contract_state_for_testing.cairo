use starknet::storage::{
    StoragePointerReadAccess, StoragePointerWriteAccess
}; // <--- Ad. 1
use testing_contract_internals::contract::Contract;
use testing_contract_internals::contract::Contract::{
    InternalTrait, _other_internal_function,
}; // <--- Ad. 2

#[test]
fn test_internal() {
    let mut state = Contract::contract_state_for_testing(); // <--- Ad. 3
    state.balance.write(10);

    let value = state._internal_get_balance();
    assert(value == 10, 'Incorrect storage value');

    let other_value = _other_internal_function(@state);
    assert(other_value == 15, 'Incorrect return value');
}
