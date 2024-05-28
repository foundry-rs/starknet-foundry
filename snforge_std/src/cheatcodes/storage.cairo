use core::array::ArrayTrait;
use core::traits::Into;
use core::option::OptionTrait;
use core::traits::TryInto;
use starknet::{testing::cheatcode, ContractAddress, storage_address_try_from_felt252};
use core::panic_with_felt252;
use super::super::_cheatcode::handle_cheatcode;

fn validate_storage_address_felt(storage_address_felt: felt252) {
    match storage_address_try_from_felt252(storage_address_felt) {
        Option::Some(_) => {},
        // Panics in order not to leave inconsistencies in the state
        Option::None(()) => panic!("storage_address out of range {}", storage_address_felt),
    }
}

fn store_felt252(target: ContractAddress, storage_address: felt252, value: felt252) {
    validate_storage_address_felt(storage_address);
    let inputs = array![target.into(), storage_address.into(), value];
    handle_cheatcode(cheatcode::<'store'>(inputs.span()));
}

fn load_felt252(target: ContractAddress, storage_address: felt252) -> felt252 {
    validate_storage_address_felt(storage_address);
    let inputs = array![target.into(), storage_address];
    *handle_cheatcode(cheatcode::<'load'>(inputs.span())).at(0)
}

/// Stores felts from `serialized_value` in `target` contract's storage, starting at
/// `storage_address`.
/// - `target` - address of the contract, which storage you want to modify
/// - `storage_address` - offset of the data in the contract's storage
/// - `serialized_value` - a sequence of felts that will be inserted starting at `storage_address`
fn store(target: ContractAddress, storage_address: felt252, serialized_value: Span<felt252>) {
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
fn load(target: ContractAddress, storage_address: felt252, size: felt252) -> Array<felt252> {
    let mut output_array: Array<felt252> = array![];
    let mut offset: usize = 0;

    while offset.into() != size {
        let loaded = load_felt252(target, storage_address + offset.into());
        output_array.append(loaded);
        offset += 1;
    };
    output_array
}

fn map_entry_address(map_selector: felt252, keys: Span<felt252>) -> felt252 {
    let mut inputs = array![map_selector];
    keys.serialize(ref inputs);
    *handle_cheatcode(cheatcode::<'map_entry_address'>(inputs.span())).at(0)
}
