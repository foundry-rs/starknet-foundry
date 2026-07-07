# `declare!`

`declare!` is a compile-time checked variant of [`declare`](./declare.md). It accepts a Cairo path instead
of a string literal and adds a compile-time check that `ContractState` exists under the given path.
The macro expands to a regular `declare(...)` call, so contract artifact resolution still happens at runtime.

> 📝 **Note**
> This compile-time check only verifies that the contract path exists as a Cairo type path.
> It does not validate the contract contents, ABI, entrypoints, or that the resolved artifact has the shape expected by the test.

Accepted paths are:
- full module tree paths (e.g. `my_package::module::MyContract`)
- partial module tree paths (e.g. `module::MyContract`)
- contract names (e.g. `MyContract`)

Contract names require the contract to be in scope, either by being defined in the same module or imported.

```rust
{{#include ../../../listings/declare_examples/tests/test_declare_macro.cairo}}
```

Partial module tree paths require their first segment to be in scope:

```rust
{{#include ../../../listings/declare_examples/tests/test_declare_macro_partial_path.cairo}}
```

### Limitations

`declare!` is type-safe only from the contract path existence perspective. It should not be treated as a
compile-time guarantee about the contract contents.

Currently, `declare!` does not resolve Cairo aliases. The path passed to the macro is also passed to the runtime
contract resolver. This means that such code will **not work**:

```rust
use my_package::module::MyContract as Alias;
use snforge_std::{ContractClassTrait, DeclareResultTrait};

#[test]
fn declare_by_alias() {
    let contract = declare!(Alias).unwrap().contract_class();
}
```

For contract names, the compile-time check uses the name visible in scope, but
runtime artifact resolution still uses the name string. If multiple contracts
share the same name, `declare!(MyContract)` can still fail due to ambiguity. Use a
module path in that case.
