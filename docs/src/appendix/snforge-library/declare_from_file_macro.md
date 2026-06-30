# `declare_from_file!`

`declare_from_file!` is a compile-time checked variant of [`declare_from_file`](./declare_from_file.md). It accepts a string literal path to a Sierra contract class JSON file and expands to a regular `declare_from_file(...)` call. The file is read, parsed and declared when the test is executed.

## Example

```rust
{{#include ../../../listings/declare_examples/tests/test_declare_from_file_macro.cairo}}
```
