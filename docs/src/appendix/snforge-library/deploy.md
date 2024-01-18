# `deploy`

```rust
trait ContractClassTrait {
    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> Result<ContractAddress, RevertedTransaction>;    
    // ...
}
```

Deploys a contract and returns its address.

- `self` - an instance of `ContractClass` struct that can be obtained by invoking [declare](./declare.md)

- `constructor_calldata` - snapshot of calldata for the constructor

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

    let contract_address = contract.deploy(@constructor_calldata).unwrap();
    // ...
}
```
