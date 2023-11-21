use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;

use snforge_std::{declare, ContractClassTrait};

use simple_package::hello_starknet::IHelloStarknetDispatcher;
use simple_package::hello_starknet::IHelloStarknetDispatcherTrait;

#[test]
fn call_and_invoke() {
    let contract = declare('HelloStarknet');
    let constructor_calldata = @ArrayTrait::new();
    let contract_address = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
