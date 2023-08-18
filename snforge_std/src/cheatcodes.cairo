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

#[derive(Drop)]
struct TxInfoMock {
    version: Option<felt252>,
    account_contract_address: Option<felt252>,
    max_fee: Option<u128>,
    signature: Option<Array<felt252>>,
    transaction_hash: Option<felt252>,
    chain_id: Option<felt252>,
    nonce: Option<felt252>,
}

trait TxInfoMockTrait {
    fn default() -> TxInfoMock;
}

impl TxInfoMockImpl of TxInfoMockTrait {
    fn default() -> TxInfoMock {
        TxInfoMock {
            version: Option::None(()),
            account_contract_address: Option::None(()),
            max_fee: Option::None(()),
            signature: Option::None(()),
            transaction_hash: Option::None(()),
            chain_id: Option::None(()),
            nonce: Option::None(()),
        }
    }
}

fn value_or_default<T, impl TDrop: Drop::<T>>(option: Option<T>, default: T) -> T {
    match option {
        Option::Some(x) => x,
        Option::None => default,
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

fn start_spoof(contract_address: ContractAddress, tx_info_mock: TxInfoMock) {
    let contract_address_felt: felt252 = contract_address.into();

    let TxInfoMock{
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce
    } = tx_info_mock;

    let mut inputs = array![
        contract_address_felt,
        value_or_default(version, 0),
        value_or_default(account_contract_address, 0),
        value_or_default(max_fee, 0_u128).into(),
        value_or_default(transaction_hash, 0),
        value_or_default(chain_id, 0),
        value_or_default(nonce, 0)
    ];

    let signature = value_or_default(signature, ArrayTrait::new());
    let signature_len = signature.len();
    inputs.append(signature_len.into());

    let mut i = 0;
    loop {
        if signature_len == i {
            break ();
        }
        inputs.append(*signature[i]);
        i += 1;
    };

    cheatcode::<'start_spoof'>(inputs.span());
}

fn stop_spoof(contract_address: ContractAddress) {
    let contract_address_felt: felt252 = contract_address.into();
    cheatcode::<'stop_spoof'>(array![contract_address_felt].span());
}
