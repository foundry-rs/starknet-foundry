# `store`

> `fn store(target: ContractAddress, storage_address: felt252, serialized_value: Span<felt252>)`

Stores felts from `serialized_value` in `target` contract's storage, starting at `storage_address`. 

- `target` - address of the contract, which storage you want to modify
- `storage_address` - offset of the data in the contract's storage 
- `serialized_value` - a sequence of felts that will be inserted starting at `storage_address` 


In order to obtain the variable address that you'd like to write to, you need to use either:
- `selector!` macro - if the variable is not a mapping
- `map_entry_address` function in tandem with `selector!` - for inserting key-value pair into a map variable

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
use snforge_std::{ store, map_entry_address };

#[test]
fn store_plain_felt() {
    // ...
    store(contract_address, selector!("plain_felt"), array![123].span());
    // plain_felt = 123
}

#[test]
fn store_map_entry() {
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
}
```

# Example: Complex structures in storage
This example uses a complex key and value, with `serde` as default serialization method.

```rust
#[starknet::contract]
mod Contract {
    #[derive(Serde)]
    struct MapKey {
        a: felt252,
        b: felt252,
    }

    // Required for indexing of complex_mapping
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
    
    // Serialization of key with `Serde` (other methods are possible)
    impl MapKeyIntoSpan of Into<MapKey, Span<felt252>> {
        fn into(self: MapKey) -> Span<felt252> {
            let mut serialized_struct: Array<felt252> = array![];
            self.serialize(ref serialized_struct);
            serialized_struct.span()
        }
    }
    
     // Serialization of value with `Serde` (other methods are possible)
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
        map_entry_address(
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
}
```

> ⚠️ **Warning**
> 
> Complex storage values or keys need to be serialized as they would be in the targeted contract's strategy.
> Using different packing (for values) or hashing (for keys) strategies might result in inconsistent contract memory.
> **Use this cheatcode as a last-resort, for cases that cannot be handled via contract's API!**

