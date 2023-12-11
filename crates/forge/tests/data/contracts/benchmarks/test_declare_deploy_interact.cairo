 use result::ResultTrait;
use traits::Into;
use starknet::ClassHashIntoFelt252;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_prank, start_roll, start_warp };

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
    let contract = declare('HelloStarknet');
    let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    dispatcher.increase_balance(100);
    let balance = dispatcher.get_balance();
    dispatcher.decrease_balance(100);
    let balance = dispatcher.get_balance();

    start_prank(CheatTarget::One(contract_address), 1234.try_into().unwrap());
    start_roll(CheatTarget::One(contract_address), 234);
    start_warp(CheatTarget::One(contract_address), 123);

    let (x, y, z) = dispatcher.interact_with_state();
}
