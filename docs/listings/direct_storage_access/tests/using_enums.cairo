use store_load::IEnumsStorageContractSafeDispatcherTrait;
use starknet::ContractAddress;
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, store, map_entry_address, load};
use store_load::{IEnumsStorageContractSafeDispatcher};

fn deploy_contract(name: ByteArray) -> ContractAddress {
    let contract = declare(name).unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
    contract_address
}


#[test]
fn test_store_and_read() {
    let contract_address = deploy_contract("EnumsStorageContract");
    let safe_dispatcher = IEnumsStorageContractSafeDispatcher { contract_address };

    let mut keys = ArrayTrait::new();
    let key: u256 = 1;
    key.serialize(ref keys);

    let value: Option<u256> = Option::Some((100));
    let felt_value: felt252 = value.unwrap().try_into().unwrap();

    // Serialize Option enum according to its 1-based storage layout
    let serialized_value = array![1, felt_value];

    let storage_address = map_entry_address(selector!("example_storage"), keys.span());

    store(
        target: contract_address,
        storage_address: storage_address,
        serialized_value: serialized_value.span(),
    );

    let read_value = safe_dispatcher.read_value(key).expect('Failed to read value');

    assert_eq!(read_value, value);
}

