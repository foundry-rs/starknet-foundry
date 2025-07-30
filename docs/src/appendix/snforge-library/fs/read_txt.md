# `read_txt`

```rust
fn read_txt(file: @File) -> Array<felt252>;
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
375902487384752430391285504918516769
436145792296873180682911121719386801692617109281
[PASS] snforge_library_reference_integrationtest::test_fs_read_txt::read_txt_example ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
