# `read_txt`

Function for reading plain text files.

```rust
fn read_txt(file: @File) -> Array<felt252>;
```

> ℹ️ **Info**
>
> Specific rules must be followed for snforge to correctly parse plain text files.
>
> Read [file format rules](./file_format_rules.md#plain-text-files) for more.

## Example

File content:
```txt
{{#include ../../../../listings/snforge_library_reference/data/hello_starknet.txt}}
```

Test code:
```rust
{{#include ../../../../listings/snforge_library_reference/tests/test_fs_read_txt.cairo}}
```

<!-- { "package_name": "snforge_library_reference" } -->
Let's run the test:
```shell
$ snforge test read_txt_example
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from snforge_library_reference package
Running 1 test(s) from tests/
0x48656c6c6f20537461726b6e657421
0x4c6574277320636f646520696e20436169726f21
0x0
0x4578616d706c652062797465206172726179
0x12
[PASS] snforge_library_reference_integrationtest::test_fs_read_txt::read_txt_example ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
