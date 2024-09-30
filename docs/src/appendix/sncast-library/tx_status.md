# `tx_status`

> `pub fn tx_status(transaction_hash: felt252) -> Result<TxStatusResult, ScriptCommandError>`

Gets the status of a transaction using its hash and returns `TxStatusResult`.

- `transaction_hash` - hash of the transaction

```rust
{{#include ../../../listings/sncast_library/scripts/tx_status/src/lib.cairo}}
```

Structures used by the command:

```rust
#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub enum FinalityStatus {
    Received,
    Rejected,
    AcceptedOnL2,
    AcceptedOnL1
}


#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub enum ExecutionStatus {
    Succeeded,
    Reverted,
}


#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub struct TxStatusResult {
    pub finality_status: FinalityStatus,
    pub execution_status: Option<ExecutionStatus>
}
```
