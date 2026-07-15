# `declare_from_file`

```rust
pub fn declare_from_file(path: ByteArray) -> Result<DeclareResult, Array<felt252>>
```

Declares a contract from a Sierra contract class JSON file, for later deployment.

The `path` argument accepts the path to the JSON file containing the Sierra contract class. Relative paths are
resolved from the package root.

If the file's class hash matches a package contract already loaded, `snforge` reuses the loaded compiled artifact
instead of compiling the Sierra file again.

Returns the [`DeclareResult`](./declare_result.md).

## Example

```rust
{{#include ../../../listings/declare_examples/tests/test_declare_from_file.cairo}}
```
