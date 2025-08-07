use starknet::{ContractAddress, StorageAddress};
use crate::cheatcode::execute_cheatcode_and_deserialize;
use crate::cheatcodes::execution_info::contract_address::{
    start_cheat_contract_address, stop_cheat_contract_address,
};

fn validate_storage_address_felt(storage_address_felt: felt252) {
    let result: Option<StorageAddress> = storage_address_felt.try_into();

    match result {
        Option::Some(_) => {},
        // Panics in order not to leave inconsistencies in the state
        Option::None(()) => panic!("storage_address out of range {}", storage_address_felt),
    }
}

fn store_felt252(target: ContractAddress, storage_address: felt252, value: felt252) {
    validate_storage_address_felt(storage_address);
    let inputs = array![target.into(), storage_address.into(), value];
    execute_cheatcode_and_deserialize::<'store', ()>(inputs.span());
}

fn load_felt252(target: ContractAddress, storage_address: felt252) -> felt252 {
    validate_storage_address_felt(storage_address);
    let inputs = array![target.into(), storage_address];
    execute_cheatcode_and_deserialize::<'load'>(inputs.span())
}

/// Stores felts from `serialized_value` in `target` contract's storage, starting at
/// `storage_address`.
/// - `target` - address of the contract, which storage you want to modify
/// - `storage_address` - offset of the data in the contract's storage
/// - `serialized_value` - a sequence of felts that will be inserted starting at `storage_address`
pub fn store(target: ContractAddress, storage_address: felt252, serialized_value: Span<felt252>) {
    let mut offset: usize = 0;
    while offset != serialized_value.len() {
        store_felt252(target, storage_address + offset.into(), *serialized_value.at(offset));
        offset += 1;
    }
}

/// Loads `size` felts from `target` contract's storage into an `Array`, starting at
/// `storage_address`.
/// - `target` - address of the contract, which storage you want to modify
/// - `storage_address` - offset of the data in the contract's storage
/// - `size` - how many felts will be loaded into the result `Array`
pub fn load(target: ContractAddress, storage_address: felt252, size: felt252) -> Array<felt252> {
    let mut output_array: Array<felt252> = array![];
    let mut offset: usize = 0;

    while offset.into() != size {
        let loaded = load_felt252(target, storage_address + offset.into());
        output_array.append(loaded);
        offset += 1;
    };
    output_array
}

pub fn map_entry_address(map_selector: felt252, keys: Span<felt252>) -> felt252 {
    let mut inputs = array![map_selector];
    keys.serialize(ref inputs);
    execute_cheatcode_and_deserialize::<'map_entry_address'>(inputs.span())
}

pub fn interact_with_state<F, +Drop<F>, impl func: core::ops::FnOnce<F, ()>, +Drop<func::Output>>(
    contract_address: ContractAddress, f: F,
) -> func::Output {
    start_cheat_contract_address(contract_address);
    let res = f();
    stop_cheat_contract_address();
    res
}
