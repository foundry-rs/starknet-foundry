use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, store, load, map_entry_address};

use direct_storage_access::complex_structures::{MapKey, MapValue};

#[test]
fn store_in_complex_mapping() {
    let (contract_address, _) = declare("ComplexStorageContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    let k = MapKey { a: 111, b: 222 };
    let v = MapValue { a: 123, b: 456 };

    store(
        contract_address,
        map_entry_address( // uses Pedersen hashing under the hood for address calculation
            selector!("mapping"), // storage variable name
            k.into() // map key
        ),
        v.into(),
    );

    // complex_mapping = {
    //     hash(k): 123,
    //     hash(k) + 1: 456
    //     ...
    // }

    let loaded = load(contract_address, map_entry_address(selector!("mapping"), k.into()), 2);

    assert_eq!(loaded, array![123, 456]);
}
