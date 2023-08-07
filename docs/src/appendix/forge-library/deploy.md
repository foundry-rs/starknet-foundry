# `deploy`

> `fn deploy(prepared_contract: PreparedContract) -> Result::<ContractAddress, RevertedTransaction>`

Deploys a contract and returns its address.

- `prepared_contract` - an instance of the struct `PreparedContract` that consists of the following fields:
  - `class_hash` - class hash of a previously [declared](declare.md) contract
  - `constructor_calldata` - snapshot of calldata for the constructor

```rust
use array::ArrayTrait;
use result::ResultTrait;
use cheatcodes::{ declare, PreparedContract, deploy };

#[test]
fn test_deploy() {
    let class_hash = declare('HelloStarknet');
    
    let mut constructor_calldata = ArrayTrait::new();
    constructor_calldata.append(42_u8.into());
    constructor_calldata.append(21);
    constructor_calldata.append(37);
  
    let prepared = PreparedContract {
        class_hash, constructor_calldata: @constructor_calldata
    };
    let contract_address = deploy(prepared).unwrap();
    // ...
}
```
