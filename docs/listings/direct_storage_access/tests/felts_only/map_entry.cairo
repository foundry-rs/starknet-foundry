use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, store, load, map_entry_address};

#[test]
fn test_store_and_load_map_entries() {
    let (contract_address, _) = declare("SimpleStorageContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    // load an existing map entry
    let loaded = load(
        contract_address,
        map_entry_address(
            selector!("mapping"), // start of the read memory chunk
            array!['some_key'].span(), // map key
        ),
        1, // length of the read memory chunk
    );

    assert_eq!(loaded, array!['some_value']);

    // write other value in place of the previous one
    store(
        contract_address,
        map_entry_address(
            selector!("mapping"), // storage variable name
             array!['some_key'].span(), // map key
        ),
        array!['some_other_value'].span()
    );

    let loaded = load(
        contract_address,
        map_entry_address(
            selector!("mapping"), // start of the read memory chunk
            array!['some_key'].span(), // map key
        ),
        1, // length of the read memory chunk
    );

    assert_eq!(loaded, array!['some_other_value']);

    // load value written under non-existing key
    let loaded = load(
        contract_address,
        map_entry_address(selector!("mapping"), array!['non_existing_field'].span(),),
        1,
    );

    assert_eq!(loaded, array![0]);
}
