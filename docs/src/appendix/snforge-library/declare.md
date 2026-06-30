# `declare`

```rust
fn declare(contract: ByteArray) -> Result<DeclareResult, Array<felt252>>
```

Declares a contract for later deployment.

The `contract` argument accepts either a contract name (e.g. `MyContract`),
an absolute module tree path (e.g. `my_package::module::MyContract`) or
a partial module tree path (e.g. `module::MyContract`).
Use the full module path to disambiguate when multiple contracts share the same name.

Returns the [`DeclareResult`](./declare_result.md).
