# `deploy_at`

```rust
trait ContractClassTrait {
    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> Result<ContractAddress, RevertedTransaction>;
    // ...
}
```

Deploys a contract at a given address and returns the address.

- `self` - an instance of `ContractClass` struct that can be obtained by invoking [declare](./declare.md)
- `constructor_calldata` - snapshot of calldata for the constructor
- `contract_address` - address the contract should be deployed at

```rust
use array::ArrayTrait;
use result::ResultTrait;
use traits::TryInto;

use snforge_std::{ declare, ContractClassTrait };


#[test]
fn test_deploy() {
    let contract = declare('HelloStarknet');

    let mut constructor_calldata = ArrayTrait::new();
    constructor_calldata.append(42_u8.into());
    constructor_calldata.append(21);
    constructor_calldata.append(37);
    
    let contract_address = 123.try_into().unwrap();

    let address = contract.deploy_at(@constructor_calldata, contract_address).unwrap();
    assert(address == contract_address, 'addresses should be the same');  // this assert passes
    // ...
}
```
