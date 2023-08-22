use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;

use starknet::testing::cheatcode;
use starknet::ClassHash;
use starknet::ContractAddress;
use starknet::ClassHashIntoFelt252;
use starknet::ContractAddressIntoFelt252;
use starknet::Felt252TryIntoClassHash;
use starknet::Felt252TryIntoContractAddress;

#[derive(Drop, Clone)]
struct ContractClass {
    class_hash: ClassHash,
}

#[derive(Drop, Clone)]
struct L1Handler {
    contract_address: ContractAddress,
    function_name: felt252,
    from_address: felt252,
    fee: u128,
    payload: Span::<felt252>,
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
        let mut inputs: Array::<felt252> = _prepare_calldata(self.class_hash, constructor_calldata);

        let outputs = cheatcode::<'precalculate_address'>(inputs.span());
        (*outputs[0]).try_into().unwrap()
    }

    fn deploy(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> Result<ContractAddress, RevertedTransaction> {
        let mut inputs = _prepare_calldata(self.class_hash, constructor_calldata);

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

fn _prepare_calldata(class_hash: @ClassHash, constructor_calldata: @Array::<felt252>) -> Array::<felt252>  {
    let class_hash: felt252 = class_hash.clone().into();
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

    inputs
}

fn declare(contract: felt252) -> ContractClass {
    let span = cheatcode::<'declare'>(array![contract].span());

    let exit_code = *span[0];
    let result = *span[1];
    assert(exit_code == 0, 'declare should never fail');
    let class_hash = result.try_into().unwrap();

    ContractClass {
        class_hash
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

trait L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler;
    fn execute(self: L1Handler);
}

impl L1HandlerImpl of L1HandlerTrait {
    fn new(contract_address: ContractAddress, function_name: felt252) -> L1Handler {
        L1Handler {
            contract_address,
            function_name,
            from_address: 0,
            fee: 1_u128,
            payload: array![].span(),
        }
    }

    fn execute(self: L1Handler) {
        let mut inputs: Array::<felt252> = array![
            self.contract_address.into(),
            self.function_name,
            self.from_address,
            self.fee.into(),
            self.payload.len().into(),
        ];

        let payload_len = self.payload.len();
        let mut i = 0;
        loop {
            if payload_len == i {
                break ();
            }
            inputs.append(*self.payload[i]);
            i += 1;
        };

        cheatcode::<'l1_handler_execute'>(inputs.span());
    }
}
