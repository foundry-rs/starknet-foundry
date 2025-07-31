# `read_json`

```rust
fn read_json(file: @File) -> Array<felt252>;
```

> ℹ️ **Info**
>
> Specific rules must be followed for snforge to correctly parse JSON and plain text files.
>
> Read more about them [here](../fs.md#file-format).

## Example

File content:
```json
{{#include ../../../../listings/snforge_library_reference/data/user.json}}
```

Test code:
```rust
{{#include ../../../../listings/snforge_library_reference/tests/test_fs_read_json.cairo}}
```

<!-- { "package_name": "snforge_library_reference" } -->
Let's run the test:
```shell
$ snforge test read_json_example
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from snforge_library_reference package
Running 1 test(s) from tests/
0x1e
0x0
0x536f66747761726520456e67696e656572
0x11
0x0
0x4e657720596f726b
0x8
0x0
0x555341
0x3
0x0
0x4a6f686e
0x4
0x0
0x446f65
0x3
[PASS] snforge_library_reference_integrationtest::test_fs_read_json::read_json_example ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
