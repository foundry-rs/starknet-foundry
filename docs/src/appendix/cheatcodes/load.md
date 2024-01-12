# `load`

> `fn load(target: ContractAddress, storage_address: felt252, size: felt252) -> Array<felt252> `

Loads a `Array` of felts from `target` contract's storage, starting at `storage_address`.

- `target` - Address of the contract, which storage you want to modify
- `storage_address` - Offset of the data in the contract's storage 
- `size` - How many felts will be loaded into the result `Array` 


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
use snforge_std::{ store, load, map_entry_address };

#[test]
fn load_plain_felt() {
    let deployed = deploy_contract();
    let loaded = load(deployed.contract_address, selector!("plain_felt"), 1);
    assert(loaded.len() == 1, 'Wrong loaded vector');
    assert(*loaded.at(0) == 0, 'Wrong loaded value');
}

#[test]
fn load_map_entry() {
    let deployed = deploy_contract();
    load(
        deployed.contract_address, 
        map_entry_address(
            selector!("mapping"), // Providing variable name
            array![123].span(),   // Providing mapping key 
        ),
        1,
    );
    
    assert(loaded.len() == 1, 'Wrong loaded vector');
    assert(*loaded.at(0) == 0, 'Wrong loaded value');
}
```

> 📝 **Note** 
> 
> The cheatcode will return 0s for uninitialized memory, which is a default storage value for starknet contracts.


> ⚠️ **Warning**
> 
> Complex data can often times be packed in a custom manner (see [this pattern](https://book.cairo-lang.org/ch99-01-03-05-optimizing-storage.html#storage-optimization-with-storepacking)) to optimize costs
> Using different lengths for packed data might result in incorrect deserialization.
> **Use this cheatcode as a last-resort, for cases that cannot be handled via contract's API!**

