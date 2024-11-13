# Direct Storage Access

In some instances, it's not possible for contracts to expose API that we'd like to use in order to initialize
the contracts before running some tests. For those cases `snforge` exposes storage-related cheatcodes,
which allow manipulating the storage directly (reading and writing).

In order to obtain the variable address that you'd like to write to, or read from, you need to use either:
- `selector!` macro - if the variable is not a mapping
- `map_entry_address` function in tandem with `selector!` - for key-value pair of a map variable
- `starknet::storage_access::storage_address_from_base`

## Example: Felt-only storage
This example uses only felts for simplicity.

1. Exact storage fields

```rust
{{#include ../../listings/snforge_advanced_features/crates/direct_storage_access/tests/felts_only/field.cairo}}
```

2. Map entries

```rust
{{#include ../../listings/snforge_advanced_features/crates/direct_storage_access/tests/felts_only/map_entry.cairo}}
```

## Example: Complex structures in storage
This example uses a complex key and value, with default derived serialization methods (via `#[derive(starknet::Store)]`).

We use a contract along with helper structs:

```rust
{{#include ../../listings/snforge_advanced_features/crates/direct_storage_access/src/complex_structures.cairo}}
```

And perform a test checking `load` and `store` behavior in context of those structs:

```rust
{{#include ../../listings/snforge_advanced_features/crates/direct_storage_access/tests/complex_structures.cairo}}
```

> ⚠️ **Warning**
>
> Complex data can often times be packed in a custom manner (see [this pattern](https://book.cairo-lang.org/ch16-01-optimizing-storage-costs.html)) to optimize costs.
> If that's the case for your contract, make sure to handle deserialization properly - standard methods might not work.
> **Use those cheatcode as a last-resort, for cases that cannot be handled via contract's API!**


> 📝 **Note**
>
> The `load` cheatcode will return zeros for memory you haven't written into yet (it is a default storage value for Starknet contracts' storage).
