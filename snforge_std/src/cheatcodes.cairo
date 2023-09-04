use array::ArrayTrait;
use array::SpanTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;

use starknet::{
    testing::cheatcode, ClassHash, ContractAddress, ClassHashIntoFelt252,
    ContractAddressIntoFelt252, Felt252TryIntoClassHash, Felt252TryIntoContractAddress
};


mod events;
mod l1_handler;
mod contract_class;
mod spoof;


fn start_roll(contract_address: ContractAddress, block_number: u64) {
    let contract_address_felt: felt252 = contract_address.into();
    let block_number_felt: felt252 = block_number.into();
    cheatcode::<'start_roll'>(array![contract_address_felt, block_number_felt].span());
}

fn stop_roll(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_roll'>(array![contract_address_felt].span());
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

fn start_warp(contract_address: ContractAddress, block_timestamp: u64) {
    let contract_address_felt: felt252 = contract_address.into();
    let block_timestamp_felt: felt252 = block_timestamp.into();
    cheatcode::<'start_warp'>(array![contract_address_felt, block_timestamp_felt].span());
}

fn stop_warp(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_warp'>(array![contract_address_felt].span());
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

fn get_class_hash(contract_address: ContractAddress) -> ClassHash {
    let contract_address_felt: felt252 = contract_address.into();

    // Expecting a buffer with one felt252, being the class hash.
    let buf = cheatcode::<'get_class_hash'>(array![contract_address_felt].span());
    (*buf[0]).try_into().expect('Invalid class hash value')
}
