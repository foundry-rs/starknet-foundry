# `declare`
```rust
#[derive(Drop, Serde, Clone)]
enum DeclareResult {
    Success: ContractClass,
    AlreadyDeclared: ContractClass,
}

trait DeclareResultTrait {
    /// Gets inner `ContractClass`
    /// `self` - an instance of the struct `DeclareResult` which is obtained by calling `declare`
    // Returns the `@ContractClass`
    fn contract_class(self: @DeclareResult) -> @ContractClass;
}

fn declare(contract: ByteArray) -> Result<DeclareResult, Array<felt252>>
```

Declares a contract for later deployment.

Returns the `DeclareResult` that encapsulated possible outcomes in the enum:
 - `Success`: Contains the successfully declared `ContractClass`.
 - `AlreadyDeclared`: Contains `ContractClass` and signals that the contract has already been declared.


See [docs of `ContractClass`](./contract_class.md) for more info about the resulting struct.
