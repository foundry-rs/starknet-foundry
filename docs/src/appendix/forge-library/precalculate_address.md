# `precalculate_address`

```rust
trait ContractClassTrait {
    fn precalculate_address(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> ContractAddress;
    // ...
}
```
Calculate the address of a contract in advance that will be returned upon the next deploy.

- `self` - an instance of the struct `ContractClass` which is obtained by calling [declare](./declare.md).

- `constructor_calldata` - snapshot of calldata for the deploy constructor. The constructor_calldata has an impact on the resulting contract_address. To precalculate the address for a future deploy, the calldata couldn't change.

```rust
use array::ArrayTrait;
use result::ResultTrait;
use cheatcodes::{ declare, ContractClassTrait };

#[test]
fn test_deploy() {
    let contract = declare('HelloStarknet');

    let mut constructor_calldata = ArrayTrait::new();
    constructor_calldata.append(42_u8.into());
    constructor_calldata.append(21);
    constructor_calldata.append(37);

    let contract_address = contract.precalculate_address(@constructor_calldata).unwrap();
    // ...
}
```
