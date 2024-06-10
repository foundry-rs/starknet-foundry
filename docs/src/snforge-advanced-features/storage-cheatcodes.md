# Direct Storage Access

In some instances, it's not possible for contracts to expose API that we'd like to use in order to initialize 
the contracts before running some tests. For those cases `snforge` exposes storage-related cheatcodes,
which allow manipulating the storage directly (reading and writing).

In order to obtain the variable address that you'd like to write to, or read from, you need to use either:
- `selector!` macro - if the variable is not a mapping
- `map_entry_address` function in tandem with `selector!` - for key-value pair of a map variable
- `starknet::storage_access::storage_address_from_base`
  
## Example: Felt-only storage
This example uses only felts for simplicity

```rust
#[starknet::contract]
mod Contract {
    #[storage]
    struct Storage {
        plain_felt: felt252,
        mapping: LegacyMap<felt252, felt252>,
    }
}

// ...
use snforge_std::{ store, load, map_entry_address };

#[test]
fn store_and_load_with_plain_felt() {
    // ...
    store(contract_address, selector!("plain_felt"), array![123].span());
    // plain_felt = 123
    let loaded = load(contract_address, selector!("plain_felt"), 1);
    assert(loaded.len() == 1, 'Wrong loaded vector');
    assert(*loaded.at(0) == 123, 'Wrong loaded value');
}


#[test]
fn store_and_load_map_entry() {
    // ...
    store(
        contract_address, 
        map_entry_address(
            selector!("mapping"), // Providing variable name
            array![123].span(),   // Providing mapping key 
        ),
        array![321].span()
    );
    
    // mapping = { 123: 321, ... }
    let loaded = load(
        contract_address, 
        map_entry_address(
            selector!("mapping"), // Providing variable name
            array![123].span(),   // Providing mapping key 
        ),
        1,
    );
    
    assert(loaded.len() == 1, 'Expected 1 felt loaded');
    assert(*loaded.at(0) == 321, 'Expected 321 value loaded');
}
```

## Example: Complex structures in storage
This example uses a complex key and value, with default derived serialization methods (via `#[derive(starknet::Store)]`).

```rust
use snforge_std::{ store, load, map_entry_address };

#[starknet::contract]
mod Contract {
    #[derive(Serde)]
    struct MapKey {
        a: felt252,
        b: felt252,
    }

    // Required for lookup of complex_mapping values
    // This is consistent with `map_entry_address`, which uses pedersen hashing of keys
    impl StructuredKeyHash of LegacyHash<MapKey> {
        fn hash(state: felt252, value: MapKey) -> felt252 {
            let state = LegacyHash::<felt252>::hash(state, value.a);
            LegacyHash::<felt252>::hash(state, value.b)
        }
    }

    #[derive(Serde, starknet::Store)]
    struct MapValue {
        a: felt252,
        b: felt252,
    }
    
    // Serialization of keys and values with `Serde` to make usage of `map_entry_address` easier 
    impl MapKeyIntoSpan of Into<MapKey, Span<felt252>> {
        fn into(self: MapKey) -> Span<felt252> {
            let mut serialized_struct: Array<felt252> = array![];
            self.serialize(ref serialized_struct);
            serialized_struct.span()
        }
    }
    impl MapValueIntoSpan of Into<MapValue, Span<felt252>> {
        fn into(self: MapValue) -> Span<felt252> {
            let mut serialized_struct: Array<felt252> = array![];
            self.serialize(ref serialized_struct);
            serialized_struct.span()
        }
    }
    
    #[storage]
    struct Storage {
        complex_mapping: LegacyMap<MapKey, MapValue>,
    }
}

// ...

#[test]
fn store_in_complex_mapping() {
    // ...
    let k = MapKey { a: 111, b: 222 };
    let v = MapValue { a: 123, b: 456 };
    
    store(
        contract_address, 
        map_entry_address(        // Uses pedersen hashing for address calculation
            selector!("mapping"), // Providing variable name
            k.into()              // Providing mapping key
        ),
        v.into()
    );
    
    // complex_mapping = { 
    //     hash(k): 123,
    //     hash(k) + 1: 456 
    //     ...
    // }
    
    let loaded = load(
        contract_address, 
        selector!("elaborate_struct"), // Providing variable name
        2,                             // Size of the struct in felts
    );
    
    assert(loaded.len() == 2, 'Expected 1 felt loaded');
    assert(*loaded.at(0) == 123, 'Expected 123 value loaded');
    assert(*loaded.at(1) == 456, 'Expected 456 value loaded');
}
```

> ⚠️ **Warning**
> 
> Complex data can often times be packed in a custom manner (see [this pattern](https://book.cairo-lang.org/ch16-01-optimizing-storage-costs.html)) to optimize costs.
> If that's the case for your contract, make sure to handle deserialization properly - standard methods might not work.
> **Use those cheatcode as a last-resort, for cases that cannot be handled via contract's API!**


> 📝 **Note** 
> 
> The `load` cheatcode will return zeros for memory you haven't written into yet (it is a default storage value for Starknet contracts' storage).


## Example with `storage_address_from_base`

This example uses `storage_address_from_base` with `address` function of the [storage variable](https://book.cairo-lang.org/ch14-01-contract-storage.html#addresses-of-storage-variables).

To retrieve storage address of a given `field`, you need to import `{field_name}ContractMemberStateTrait` from the contract.

```rust
#[starknet::contract]
mod Contract {
    #[storage]
    struct Storage {
        map: LegacyMap::<(u8, u32), u32>,
    }
}

// ...
use starknet::storage_access::storage_address_from_base;
use snforge_std::{ store, load };
use Contract::mapContractMemberStateTrait;

#[test]
fn update_mapping() {
    let key = (1_u8, 10_u32);
    let data = 42_u32;

    // ...
    let mut state = Contract::contract_state_for_testing();
    let storage_address: felt252 = storage_address_from_base(
        state.map.address(key)
    )
    .into();
    let storage_value: Span<felt252> = array![data.into()].span();
    store(contract_address, storage_address, storage_value);

    let read_data: u32 = load(contract_address, storage_address, 1).at(0).try_into().unwrap():
    assert_eq!(read_data, data, "Storage update failed")
}

```

