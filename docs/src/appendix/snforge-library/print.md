# `print`

> `trait PrintTrait<T> { fn print(self: T); }`

Trait used for displaying test data with the `snforge` command line output.

The trait is implemented for types:

- `felt252`
- `Array<felt252>`
- `ContractAddress`
- `u8`, `u16`, `u32`, `u64`, `u128`, `u256`
- `i8`, `i16`, `i32`, `i64`, `i128`
- `bool`

`snforge` will attempt to convert these values to strings and display them.
In case the value contains parts that evaluate to ascii control characters (e.g. 27: `ESC`), the value will not be
printed.

```rust
use array::ArrayTrait;
use snforge_std::PrintTrait;
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
