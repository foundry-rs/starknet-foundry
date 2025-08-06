use contract_state::balance::HelloStarknetExtended::InternalFunctionsTrait;
use contract_state::balance::{
    HelloStarknetExtended, IHelloStarknetExtendedDispatcher, IHelloStarknetExtendedDispatcherTrait,
};
use snforge_std::interact_with_state;
use starknet::ContractAddress;
use starknet::storage::{
    MutableVecTrait, StorageMapWriteAccess, StoragePointerReadAccess, StoragePointerWriteAccess,
};
use crate::utils::deploy_contract;

#[test]
fn test_interact_with_state() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);
    let dispatcher = IHelloStarknetExtendedDispatcher { contract_address };

    assert(dispatcher.get_balance() == 0, 'Wrong balance');

    interact_with_state(
        contract_address,
        || {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            state.balance.write(987);
        },
    );

    assert(dispatcher.get_balance() == 987, 'Wrong balance');
    dispatcher.increase_balance(13);
    assert(dispatcher.get_balance() == 1000, 'Wrong balance');
}

#[test]
fn test_interact_with_state_return() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);
    let dispatcher = IHelloStarknetExtendedDispatcher { contract_address };

    assert(dispatcher.get_balance() == 0, 'Wrong balance');

    let res = interact_with_state(
        contract_address,
        || -> u256 {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            state.balance.write(111);
            state.balance.read()
        },
    );

    assert(res == 111, 'Wrong balance');
}

#[test]
fn test_interact_with_initialized_state() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);
    let dispatcher = IHelloStarknetExtendedDispatcher { contract_address };

    dispatcher.increase_balance(199);

    interact_with_state(
        contract_address,
        || {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            assert(state.balance.read() == 199, 'Wrong balance');
            state.balance.write(1);
        },
    );

    assert(dispatcher.get_balance() == 1, 'Wrong balance');
}

#[test]
fn test_interact_with_state_vec() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);
    let dispatcher = IHelloStarknetExtendedDispatcher { contract_address };

    dispatcher.increase_balance(1);
    dispatcher.increase_balance(1);
    dispatcher.increase_balance(1);

    interact_with_state(
        contract_address,
        || {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            assert(state.balance_records.len() == 4, 'Wrong length');
            state.balance_records.push(10);
        },
    );

    assert(dispatcher.get_balance_at(0) == 0, 'Wrong balance');
    assert(dispatcher.get_balance_at(2) == 2, 'Wrong balance');
    assert(dispatcher.get_balance_at(4) == 10, 'Wrong balance');
}

#[test]
fn test_interact_with_state_map() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);
    let dispatcher = IHelloStarknetExtendedDispatcher { contract_address };

    dispatcher.increase_balance(1);

    interact_with_state(
        contract_address,
        || {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            state.callers.write(0x123.try_into().unwrap(), 1000);
            state.callers.write(0x321.try_into().unwrap(), 2000);
        },
    );

    assert(
        dispatcher.get_caller_info(0x123.try_into().unwrap()) == 1000,
        'Wrong data for address 0x123',
    );
    assert(
        dispatcher.get_caller_info(0x321.try_into().unwrap()) == 2000,
        'Wrong data for address 0x321',
    );
    assert(
        dispatcher.get_caller_info(0x12345.try_into().unwrap()) == 0,
        'Wrong data for address 0x12345',
    );
}

#[test]
fn test_interact_with_state_internal_function() {
    let contract_address = deploy_contract("HelloStarknetExtended", array!['Name']);

    let get_owner = || -> (ContractAddress, felt252) {
        interact_with_state(
            contract_address,
            || -> (ContractAddress, felt252) {
                let mut state = HelloStarknetExtended::contract_state_for_testing();
                (state.owner.address.read(), state.owner.name.read())
            },
        )
    };
    let (owner_address, owner_name) = get_owner();
    assert(owner_address == 0.try_into().unwrap(), 'Incorrect owner address');
    assert(owner_name == 'Name', 'Incorrect owner name');

    interact_with_state(
        contract_address,
        || {
            let mut state = HelloStarknetExtended::contract_state_for_testing();
            state._set_owner(0x777.try_into().unwrap(), 'New name');
        },
    );
    let (owner_address, owner_name) = get_owner();

    assert(owner_address == 0x777.try_into().unwrap(), 'Incorrect owner address');
    assert(owner_name == 'New name', 'Incorrect owner name');
}
