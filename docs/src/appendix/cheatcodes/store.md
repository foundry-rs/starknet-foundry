# `store`

> `fn store(target: ContractAddress, storage_address: felt252, serialized_value: Span<felt252>)`

Stores felts from `serialized_value` in `target` contract's storage, starting at `storage_address`. 

- `target` - address of the contract, which storage you want to modify
- `storage_address` - offset of the data in the contract's storage 
- `serialized_value` - a sequence of felts that will be inserted starting at `storage_address` 


In order to obtain the variable address that you'd like to write to, you need to use either:
- `selector!` macro - if the variable is not a mapping
- `map_entry_address` function in tandem with `selector!` - for inserting key-value pair into a map variable

## Example usage

```rust
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

> ⚠️ **Warning**
> 
> Complex storage values or keys need to be serialized as they would be in the targeted contract's strategy.
> Using different packing (for values) or hashing (for keys) strategies might result in inconsistent contract memory.
> **Use this cheatcode as a last-resort, for cases that cannot be handled via contract's API!**

