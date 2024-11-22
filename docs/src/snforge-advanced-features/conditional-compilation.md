# Conditional Compilation

> 📝 **Note**
> For more detailed guide on Scarb conditional compilation, please refer to [Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/conditional-compilation.html)


It is possible to build some contracts solely for testing purposes.
This can be achieved by leveraging [Scarb features](https://docs.swmansion.com/scarb/docs/reference/conditional-compilation.html#features).
Configuration in `Scarb.toml` is done in the same manner as described in the Scarb documentation.
Additionally, for utilizing features the `snforge test` command exposes the following flags, aligned with `scarb` flags:
[--features](../appendix/snforge/test.md#-f---features-features),
[--all-features](../appendix/snforge/test.md#--all-features) and [--no-default-features](../appendix/snforge/test.md#--no-default-features).

## Contracts

Firstly, define a contract in the `src` directory with a `#[cfg(feature: '<FEATURE_NAME>')]` attribute:

```rust
{{#include ../../listings/snforge_advanced_features/crates/conditional_compilation/src/lib.cairo}}
```

> 📝 **Note**
> To declare mock contracts in tests, these contracts should be defined within the package and not in the `tests` directory.
> This requirement is due to the way snforge [collects contracts](../testing/contracts-collection.md).


Next, create a test that uses the above contract:

```rust
{{#include ../../listings/snforge_advanced_features/crates/conditional_compilation/tests/test.cairo}}
```

The `Scarb.toml` file needs to be updated so it includes the following lines:

```toml
[features]
enable_for_tests = []
```

Then, to use the contract in tests `snforge test` must be provided with a flag defined above:

```shell
$ snforge test --features enable_for_tests
```

Also, we can specify which features are going to be enabled by default:

```toml
[features]
default = ["enable_for_tests"]
enable_for_tests = []
```

> 📝 **Note**
> If `snforge test` is run without the above feature enabled, it won't build any artifacts for the `MockContract` and all tests that use this contract will fail.

## Functions

Features are not limited to conditionally compiling contracts and can be used with other parts of the code, like functions:

```rust
{{#include ../../listings/snforge_advanced_features/crates/conditional_compilation/src/function.cairo}}
```
