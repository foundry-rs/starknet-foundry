# `DeclareResult`

```rust
#[derive(Drop, Serde, Clone)]
pub enum DeclareResult {
    Success: ContractClass,
    AlreadyDeclared: ContractClass,
}

pub trait DeclareResultTrait {
    /// Gets inner `ContractClass`
    /// `self` - an instance of the struct `DeclareResult` which is obtained by calling `declare`
    // Returns the `@ContractClass`
    fn contract_class(self: @DeclareResult) -> @ContractClass;
}
```


Encapsulated possible outcomes of contract declaration in the enum:
 - `Success`: Contains the successfully declared `ContractClass`.
 - `AlreadyDeclared`: Contains `ContractClass` and signals that the contract has already been declared.


See [docs of `ContractClass`](./contract_class.md) for more info about the resulting struct.
