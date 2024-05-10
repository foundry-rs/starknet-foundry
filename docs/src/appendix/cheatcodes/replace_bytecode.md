# `replace_bytecode`

> `fn replace_bytecode(contract: ContractAddress, new_class: ClassHash) -> Result<(), ReplaceBytecodeError>`

Replaces class for given contract address.
The `new_class` hash has to be declared in order for the replacement class to execute the code when interacting with the contract.
Returns `Result::Ok` if the replacement succeeded, and a `ReplaceBytecodeError` with appropriate error type otherwise

## ReplaceBytecodeError
An enum with appropriate type of replacement failure

```rust
pub enum ReplaceBytecodeError {
    /// Means that the contract does not exist, and thus bytecode cannot be replaced
    ContractNotDeployed,
}
```