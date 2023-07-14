# `print`

> `trait PrintTrait<T> { fn print(self: T); }`

Trait used for displaying data.

The trait is implemented for types:
- `felt252`
- `Array<felt252>`
- `ContractAddress`
- `u8`, `u64`, `u128`, `u256`
- `bool`

```rust
use array::ArrayTrait;
use forge_print::PrintTrait;
use starknet::contract_address_const;

#[test]
fn test() {
    //...
    1.print();
    
    let mut calldata = ArrayTrait::<felt252>::new();
    calldata.append(42);
    calldata.append(21);
    calldata.print();
    
    true.print();
    
    let contract_address = contract_address_const::<37>();
    contract_address.print();
    // ...
}
```
