# `tx_status`

> `pub fn tx_status(transaction_hash: felt252) -> Result<TxStatusResult, ScriptCommandError>`

Gets the status of a transaction using its hash and returns `TxStatusResult`.

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

- `transaction_hash` - hash of the transaction

```rust
use sncast_std::{tx_status};

fn main() {
    let transaction_hash = 0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c;
    let status = tx_status(transaction_hash);

    match status {
        Result::Ok(TxStatusResult) => println!("transaction status: {:?}", TxStatusResult),
        Result::Err(ScriptCommandError) => println!("error occurred: {:?}", ScriptCommandError),
    }
}
```