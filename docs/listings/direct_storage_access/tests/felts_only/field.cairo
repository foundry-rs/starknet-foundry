use snforge_std::{ContractClassTrait, DeclareResultTrait, declare, load, map_entry_address, store};

#[test]
fn test_store_and_load_plain_felt() {
    let (contract_address, _) = declare("SimpleStorageContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    // load existing value from storage
    let loaded = load(
        contract_address, // an existing contract which owns the storage
        selector!("plain_felt"), // field marking the start of the memory chunk being read from
        1 // length of the memory chunk (seen as an array of felts) to read
    );

    assert_eq!(loaded, array![0x2137_felt252]);

    // overwrite it with a new value
    store(
        contract_address, // storage owner
        selector!("plain_felt"), // field marking the start of the memory chunk being written to
        array![420].span() // array of felts to write
    );

    // load again and check if it changed
    let loaded = load(contract_address, selector!("plain_felt"), 1);
    assert_eq!(loaded, array![420]);
}
