# `precalculate_address`

> ⚠️ **Warning**
>
> Precalculated address is only correct for the very next [`deploy`](./deploy.md) call. It will be different for all `deploy` calls after the first one after calculating the address.


```rust
trait ContractClassTrait {
    fn precalculate_address(self: @ContractClass, constructor_calldata: @Array::<felt252>) -> ContractAddress;
    // ...
}
```
Calculate an address of a contract in advance that would be returned when calling [`deploy`](./deploy.md).

- `self` - an instance of the struct `ContractClass` which is obtained by calling [`declare`](./declare.md).

- `constructor_calldata` - snapshot of calldata for the deploy constructor. The `constructor_calldata` has an impact on the resulting contract address. To precalculate an address for a future deployment, the calldata cannot change.

```rust
use array::ArrayTrait;
use result::ResultTrait;
use snforge_std::{ declare, ContractClassTrait };

#[test]
fn test_deploy() {
    let contract = declare('HelloStarknet');

    let mut constructor_calldata = ArrayTrait::new();
    constructor_calldata.append(42_u8.into());
    constructor_calldata.append(21);
    constructor_calldata.append(37);

    let contract_address = contract.precalculate_address(@constructor_calldata);
    // ...
}
```
