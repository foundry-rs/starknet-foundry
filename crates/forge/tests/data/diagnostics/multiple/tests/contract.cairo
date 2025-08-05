use core::array::ArrayTrait;
use core::result::ResultTrait;
use multiple::{IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare};

#[test]
#[fuzzer]
#[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
#[ignore]
fn call_and_invoke(_a: felt252, b: u256) {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    // Error below
    assert(balance === 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}

#[test]
#[fuzzer]
#[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
#[ignore]
fn call_and_invoke2(_a: felt252, b: u256) {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}

#[test]
#[fuzzer]
#[fork(url: "http://127.0.0.1:3030", block_tag: latest)]
#[ignore]
fn call_and_invoke3(_a: felt252, b: u256) {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    // Error below
    let balance = dispatcher/get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
