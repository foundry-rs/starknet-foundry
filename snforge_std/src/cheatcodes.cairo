use array::ArrayTrait;
use array::SpanTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;

use starknet::{
    testing::cheatcode, ClassHash, ContractAddress, ClassHashIntoFelt252,
    ContractAddressIntoFelt252, Felt252TryIntoClassHash, Felt252TryIntoContractAddress,
    contract_address_const
};


mod events;
mod l1_handler;
mod contract_class;
mod tx_info;
mod fork;

#[derive(Drop, Serde)]
enum CheatTarget {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}

fn test_address() -> ContractAddress {
    contract_address_const::<469394814521890341860918960550914>()
}

fn start_roll(target: CheatTarget, block_number: u64) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    inputs.append(block_number.into());
    cheatcode::<'start_roll'>(inputs.span());
}

fn stop_roll(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    cheatcode::<'stop_roll'>(inputs.span());
}

fn start_prank(contract_address: ContractAddress, caller_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    let caller_address_felt: felt252 = caller_address.into();
    cheatcode::<'start_prank'>(array![contract_address_felt, caller_address_felt].span());
}

fn stop_prank(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_prank'>(array![contract_address_felt].span());
}

fn start_warp(target: CheatTarget, block_number: u64) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    inputs.append(block_number.into());
    cheatcode::<'start_warp'>(inputs.span());
}

fn stop_warp(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    cheatcode::<'stop_warp'>(inputs.span());
}

fn start_mock_call<T, impl TSerde: serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress, function_name: felt252, ret_data: T
) {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_name];

    let mut ret_data_arr = ArrayTrait::new();
    ret_data.serialize(ref ret_data_arr);

    let ret_data_len = ret_data_arr.len();

    inputs.append(ret_data_len.into());

    let mut i = 0;
    loop {
        if ret_data_len == i {
            break ();
        }
        inputs.append(*ret_data_arr[i]);
        i += 1;
    };

    cheatcode::<'start_mock_call'>(inputs.span());
}

fn stop_mock_call(contract_address: ContractAddress, function_name: felt252) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_mock_call'>(array![contract_address_felt, function_name].span());
}
