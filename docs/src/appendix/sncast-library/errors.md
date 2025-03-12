# `errors`

```rust
#[derive(Drop, PartialEq, Serde, Debug)]
pub struct ErrorData {
    msg: ByteArray
}

#[derive(Drop, PartialEq, Serde, Debug)]
pub struct TransactionExecutionErrorData {
    transaction_index: felt252,
    execution_error: ByteArray,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum StarknetError {
    /// Failed to receive transaction
    FailedToReceiveTransaction,
    /// Contract not found
    ContractNotFound,
    /// Block not found
    BlockNotFound,
    /// Invalid transaction index in a block
    InvalidTransactionIndex,
    /// Class hash not found
    ClassHashNotFound,
    /// Transaction hash not found
    TransactionHashNotFound,
    /// Contract error
    ContractError: ErrorData,
    /// Transaction execution error
    TransactionExecutionError: TransactionExecutionErrorData,
    /// Class already declared
    ClassAlreadyDeclared,
    /// Invalid transaction nonce
    InvalidTransactionNonce,
    /// The transaction's resources don't cover validation or the minimal transaction fee
    InsufficientResourcesForValidate,
    /// Account balance is smaller than the transaction's max_fee
    InsufficientAccountBalance,
    /// Account validation failed
    ValidationFailure: ErrorData,
    /// Compilation failed
    CompilationFailed,
    /// Contract class size it too large
    ContractClassSizeIsTooLarge,
    /// Sender address in not an account contract
    NonAccount,
    /// A transaction with the same hash already exists in the mempool
    DuplicateTx,
    /// the compiled class hash did not match the one supplied in the transaction
    CompiledClassHashMismatch,
    /// the transaction version is not supported
    UnsupportedTxVersion,
    /// the contract class version is not supported
    UnsupportedContractClassVersion,
    /// An unexpected error occurred
    UnexpectedError: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ProviderError {
    StarknetError: StarknetError,
    RateLimited,
    UnknownError: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum TransactionError {
    Rejected,
    Reverted: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum WaitForTransactionError {
    TransactionError: TransactionError,
    TimedOut,
    ProviderError: ProviderError,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ScriptCommandError {
    UnknownError: ErrorData,
    ContractArtifactsNotFound: ErrorData,
    WaitForTransactionError: WaitForTransactionError,
    ProviderError: ProviderError,
}
```