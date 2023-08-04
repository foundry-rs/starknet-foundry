use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;
use traits::Into;
use traits::TryInto;

use starknet::testing::cheatcode;
use starknet::ClassHash;
use starknet::ContractAddress;
use starknet::ClassHashIntoFelt252;
use starknet::ContractAddressIntoFelt252;
use starknet::Felt252TryIntoClassHash;
use starknet::Felt252TryIntoContractAddress;

#[derive(Drop, Clone)]
struct PreparedContract {
    class_hash: ClassHash,
    constructor_calldata: @Array::<felt252>,
}

#[derive(Drop, Clone)]
struct RevertedTransaction {
    panic_data: Array::<felt252>,
}

trait RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252;
}

impl RevertedTransactionImpl of RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252 {
        *self.panic_data.at(0)
    }
}

fn declare(contract: felt252) -> ClassHash {
    let span = cheatcode::<'declare'>(array![contract].span());

    let exit_code = *span[0];
    let result = *span[1];
    assert(exit_code == 0, 'declare should never fail');
    result.try_into().unwrap()
}

fn deploy(prepared_contract: PreparedContract) -> Result::<ContractAddress, RevertedTransaction> {
    let PreparedContract{class_hash, constructor_calldata } = prepared_contract;
    let mut inputs = array![class_hash.into()];

    let calldata_len_felt = constructor_calldata.len().into();
    inputs.append(calldata_len_felt);

    let calldata_len = constructor_calldata.len();
    let mut i = 0;
    loop {
        if calldata_len == i {
            break ();
        }
        inputs.append(*constructor_calldata[i]);
        i += 1;
    };

    let outputs = cheatcode::<'deploy'>(inputs.span());
    let exit_code = *outputs[0];

    if exit_code == 0 {
        let result = *outputs[1];
        Result::<ContractAddress, RevertedTransaction>::Ok(result.try_into().unwrap())
    } else {
        let panic_data_len_felt = *outputs[1];
        let panic_data_len = panic_data_len_felt.try_into().unwrap();
        let mut panic_data = array![];

        let offset = 2;
        let mut i = offset;
        loop {
            if panic_data_len + offset == i {
                break ();
            }
            panic_data.append(*outputs[i]);
            i += 1;
        };

        Result::<ContractAddress, RevertedTransaction>::Err(RevertedTransaction { panic_data })
    }
}

fn start_roll(contract_address: ContractAddress, block_number: u64) {
    let contract_address_felt: felt252 = contract_address.into();
    let block_number_felt: felt252 = block_number.into();
    cheatcode::<'start_roll'>(array![contract_address_felt, block_number_felt].span());
}

fn start_prank(contract_address: ContractAddress, caller_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    let caller_address_felt: felt252 = caller_address.into();
    cheatcode::<'start_prank'>(array![contract_address_felt, caller_address_felt].span());
}

fn start_warp(contract_address: ContractAddress, block_timestamp: u64) {
    let contract_address_felt: felt252 = contract_address.into();
    let block_timestamp_felt: felt252 = block_timestamp.into();
    cheatcode::<'start_warp'>(array![contract_address_felt, block_timestamp_felt].span());
}

fn stop_roll(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_roll'>(array![contract_address_felt].span());
}

fn stop_prank(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_prank'>(array![contract_address_felt].span());
}

fn stop_warp(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_warp'>(array![contract_address_felt].span());
}

fn start_mock_call(contract_address: ContractAddress, fn_name: felt252, ret_data: Array::<felt252>) {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, fn_name];

    let ret_data_len = ret_data.len();

    inputs.append(ret_data_len.into());

    let mut i = 0;
    loop {
        if ret_data_len == i {
            break ();
        }
        inputs.append(*ret_data[i]);
        i += 1;
    };

    cheatcode::<'start_mock_call'>(inputs.span());
}

fn stop_mock_call(contract_address: ContractAddress, fn_name: felt252) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_mock_call'>(array![contract_address_felt, fn_name].span());
}
