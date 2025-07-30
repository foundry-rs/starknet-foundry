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
30
0
28391512738467412385612170632190008583538
17
0
5649052288429290091
8
0
5591873
3
0
1248815214
4
0
4484965
3
[PASS] snforge_library_reference_integrationtest::test_fs_read_json::read_json_example ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
