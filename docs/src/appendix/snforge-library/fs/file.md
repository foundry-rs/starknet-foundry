# `File`

```rust
trait FileTrait {
    fn new(path: ByteArray) -> File;
}
```

> ℹ️ **Info**
>
> Specific rules must be followed for snforge to correctly parse JSON and plain text files.
>
> Read more about them [here](../fs.md#file-format).

## Example

File content:
```txt
{{#include ../../../../listings/snforge_library_reference/data/hello_starknet.txt}}
```

Test code:
```rust
{{#include ../../../../listings/snforge_library_reference/tests/test_fs_file_trait.cairo}}
```
