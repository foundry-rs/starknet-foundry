# `declare`

> `pub fn declare(contract_name: ByteArray, fee_settings: FeeSettings, nonce: Option<felt252>) -> Result<DeclareResult, ScriptCommandError>`

Declares a contract and returns `DeclareResult`.

- `contract_name` - name of a contract as Cairo string. It is a name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.
- `fee_settings` - fee settings for the transaction.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/declare/src/lib.cairo}}
```

## Returned Type

* If the contract has not been declared, `DeclareResult::Success` is returned containing respective transaction hash.
* If the contract has already been declared, `DeclareResult::AlreadyDeclared` is returned.

## Getting the Class Hash

Both variants contain `class_hash` of the declared contract. Import `DeclareResultTrait` to access it.

```rust
pub trait DeclareResultTrait {
    fn class_hash(self: @DeclareResult) -> @ClassHash;
}
```

## Structures Used by the Command

```rust
#[derive(Drop, Copy, Debug, Serde)]
pub enum DeclareResult {
    Success: DeclareTransactionResult,
    AlreadyDeclared: AlreadyDeclaredResult,
}

#[derive(Drop, Copy, Debug, Serde)]
pub struct DeclareTransactionResult {
    pub class_hash: ClassHash,
    pub transaction_hash: felt252,
}

#[derive(Drop, Copy, Debug, Serde)]
pub struct AlreadyDeclaredResult {
    pub class_hash: ClassHash,
}
#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub struct FeeSettings {
    pub max_fee: Option<felt252>,
    pub max_gas: Option<u64>,
    pub max_gas_unit_price: Option<u128>,
}
```
