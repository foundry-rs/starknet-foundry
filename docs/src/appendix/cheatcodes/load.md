# `load`

> `fn load(target: ContractAddress, storage_address: felt252, size: felt252) -> Array<felt252> `

Loads `size` felts from `target` contract's storage into an `Array`, starting at `storage_address`.

- `target` - address of the contract, which storage you want to modify
- `storage_address` - offset of the data in the contract's storage 
- `size` - how many felts will be loaded into the result `Array` 


## Example usage

```rust
mod Contract {
    #[storage]
    struct Storage {
        plain_felt: felt252,
        elaborate_struct: ElaborateStruct,
        mapping: LegacyMap<felt252, felt252>,
    }
}

// ...
use snforge_std::{ load, map_entry_address };

#[test]
fn load_plain_felt() {
    // ...
    
    let loaded = load(contract_address, selector!("plain_felt"), 1);
    assert(loaded.len() == 1, 'Wrong loaded vector');
    assert(*loaded.at(0) == 0, 'Wrong loaded value');
}

#[test]
fn load_map_entry() {
    // ...
    let loaded = load(
        contract_address, 
        map_entry_address(
            selector!("mapping"), // Providing variable name
            array![123].span(),   // Providing mapping key 
        ),
        1,
    );
    
    assert(loaded.len() == 1, 'Expected 1 felt loaded');
    assert(*loaded.at(0) == 0, 'Expected 0 value loaded');
}

// Generates `impl StoreElaborateStructure of starknet::Store<ElaborateStructure>` needed for size retrieval 
#[derive(starknet::Store)]
struct ElaborateStructure {
    a: felt252,
    b: felt252,
    // ...
}

#[test]
fn load_elaborate_struct() {
    // ...
    let loaded = load(
        contract_address, 
        selector!("elaborate_struct"), // Providing variable name
        StoreElaborateStructure::size().into(), // using generated StoreElaborateStructure impl to get struct size
    );
    
    assert(loaded.len() == 1, 'Expected 1 felt loaded');
    assert(*loaded.at(0) == 0, 'Expected 0 value loaded');
}
```

> 📝 **Note** 
> 
> The cheatcode will return zeros for memory you haven't written into yet (it is a default storage value for Starknet contracts' storage).


> ⚠️ **Warning**
> 
> Complex data can often times be packed in a custom manner (see [this pattern](https://book.cairo-lang.org/ch99-01-03-05-optimizing-storage.html#storage-optimization-with-storepacking)) to optimize costs.
> If that's the case for your contract, make sure to handle deserialization properly - standard methods might not work.
> **Use this cheatcode as a last-resort, for cases that cannot be handled via contract's API!**

