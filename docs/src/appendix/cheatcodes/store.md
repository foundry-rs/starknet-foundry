# `store`

> `fn store(target: ContractAddress, storage_address: felt252, serialized_value: Span<felt252>)`

Stores a `Span` of felts in `target` contract's storage, starting at `storage_address`. 

- `target` - Address of the contract, which storage you want to modify
- `storage_address` - Offset of the data in the contract's storage 
- `serialized_value` - A sequence of felts that will be inserted starting at `storage_address` 


In order to obtain the variable address that you'd like to write to, you need to use either:
- `selector!` macro - if the variable is a plain variable
- `map_entry_address` function in tandem with `selector!` - if the variable is a mapping, and you want to insert a key-value pair

Those examples present the usage in practise:

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
    let deployed = deploy_contract();
    store(deployed.contract_address, selector!("plain_felt"), array![123].span());
}

#[test]
fn store_map_entry() {
    let deployed = deploy_contract();
    store(
        deployed.contract_address, 
        map_entry_address(
            selector!("mapping"), // Providing variable name
            array![123].span(),   // Providing mapping key 
        ),
        array![321].span()
    );
}
```

> ⚠️ **Warning**
> 
> Complex storage values or keys need to be serialized as they would be in the targeted contract's strategy.
> Using different packing (for values) or hashing (for keys) strategies might result in inconsistent contract memory.
> **Use this cheatcode as a last-resort, for cases that cannot be handled via contract's API!**

