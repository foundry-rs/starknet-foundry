# `declare_from_file!`

`declare_from_file!` is a compile-time checked variant of [`declare_from_file`](./declare_from_file.md). It accepts a string literal path to a Sierra contract class JSON file and validates that the file can be read and parsed during macro expansion.

## Example

```rust
{{#include ../../../listings/declare_examples/tests/test_declare_from_file_macro.cairo}}
```
