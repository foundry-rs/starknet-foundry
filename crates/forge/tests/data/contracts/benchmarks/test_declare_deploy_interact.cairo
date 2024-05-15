use result::ResultTrait;
use traits::Into;
use starknet::ClassHashIntoFelt252;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use snforge_std::{declare, ContractAddress, ContractClassTrait, start_cheat_caller_address, start_cheat_block_number, start_cheat_block_timestamp};

#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn decrease_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn interact_with_state(self: @TContractState) -> (felt252, felt252, felt252);
}

#[test]
fn declare_and_interact() {
    assert(1 == 1, 'simple check');
    let contract = declare("HelloStarknet").unwrap();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    dispatcher.get_balance();
    dispatcher.increase_balance(100);
    dispatcher.get_balance();
    dispatcher.decrease_balance(100);
    dispatcher.get_balance();

    start_cheat_caller_address(ContractAddress::One(contract_address), 1234.try_into().unwrap());
    start_cheat_block_number(ContractAddress::One(contract_address), 234);
    start_cheat_block_timestamp(ContractAddress::One(contract_address), 123);

    dispatcher.interact_with_state();
}
