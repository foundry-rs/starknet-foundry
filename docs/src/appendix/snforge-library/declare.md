# `declare` and `declare!`
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

fn declare(contract: ByteArray) -> Result<DeclareResult, Array<felt252>>
```

Declares a contract for later deployment.

The `contract` argument accepts either a contract name (e.g. `MyContract`),
an absolute module tree path (e.g. `my_package::module::MyContract`) or
a partial module tree path (e.g. `module::MyContract`).
Use the full module path to disambiguate when multiple contracts share the same name.

Returns the `DeclareResult` that encapsulated possible outcomes in the enum:
 - `Success`: Contains the successfully declared `ContractClass`.
 - `AlreadyDeclared`: Contains `ContractClass` and signals that the contract has already been declared.


See [docs of `ContractClass`](./contract_class.md) for more info about the resulting struct.

## Type-safe `declare!` macro

`declare!` is a type-safe variant of `declare`. It accepts a Cairo path instead
of a string literal and expands to a regular `declare(...)` call with an
additional compile-time check that `ContractState` exists under the given path.

Accepted paths are:
- full module tree paths (e.g. `my_package::module::MyContract`)
- partial module tree paths (e.g. `module::MyContract`)
- contract names (e.g. `MyContract`)

Contract names require the contract to be in scope, either by being defined in the same module or imported.

```rust
use my_package::module::MyContract;
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};

#[test]
fn declare_imported_contract() {
    let contract = declare!(MyContract).unwrap().contract_class();
}
```

Partial module tree paths require their first segment to be in scope:

```rust
use my_package::module;
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};

#[test]
fn declare_by_partial_path() {
    let contract = declare!(module::MyContract).unwrap().contract_class();
}
```
