use starknet::storage::StorageAsPointer;
use starknet::storage::StoragePathEntry;

use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, store, load};
use starknet::storage_access::{storage_address_from_base};

use direct_storage_access::felts_only::{SimpleStorageContract};

#[test]
fn update_mapping() {
    let key = 0;
    let data = 100;
    let (contract_address, _) = declare("SimpleStorageContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();
    let mut state = SimpleStorageContract::contract_state_for_testing();

    let storage_address = storage_address_from_base(
        state.mapping.entry(key).as_ptr().__storage_pointer_address__.into()
    );
    let storage_value: Span<felt252> = array![data.into()].span();
    store(contract_address, storage_address.into(), storage_value);

    let read_data = load(contract_address, storage_address.into(), 1)
        .at(key.try_into().unwrap())
        .try_into()
        .unwrap();

    assert_eq!(read_data, @data, "Storage update failed")
}
