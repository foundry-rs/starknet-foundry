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
struct ContractClass {
    class_hash: ClassHash,
}

#[derive(Drop, Clone)]
struct RevertedTransaction {
    panic_data: Array::<felt252>,
}

trait ContractClassTrait {
    fn precalculate_address(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> ContractAddress;
    fn deploy(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> Result<ContractAddress, RevertedTransaction>;
}

trait RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252;
}

impl RevertedTransactionImpl of RevertedTransactionTrait {
    fn first(self: @RevertedTransaction) -> felt252 {
        *self.panic_data.at(0)
    }
}

impl ContractClassImpl of ContractClassTrait {
    fn precalculate_address(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> ContractAddress {
        let class_hash: felt252 = (*self.class_hash).into();
        let mut inputs: Array::<felt252> = array![class_hash];
        let calldata_len_felt = constructor_calldata.len().into();
        inputs.append(calldata_len_felt);

        let calldata_len = constructor_calldata.len();
        let mut i = 0;

        loop {
            if i == calldata_len {
                break();
            }
            inputs.append(*constructor_calldata[i]);
            i += 1;
        };

        let outputs = cheatcode::<'precalculate_address'>(inputs.span());
        let result = *outputs[0];
        result.try_into().unwrap()
    }
    fn deploy(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> Result<ContractAddress, RevertedTransaction> {
        let class_hash: felt252 = (*self.class_hash).into();
        let mut inputs = array![class_hash];

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
}

fn declare(contract: felt252) -> ContractClass {
    let span = cheatcode::<'declare'>(array![contract].span());

    let exit_code = *span[0];
    let result = *span[1];
    assert(exit_code == 0, 'declare should never fail');
    let class_hash = result.try_into().unwrap();

    ContractClass {
        class_hash: class_hash
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
