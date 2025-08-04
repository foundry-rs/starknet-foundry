# `FileParser<T>`

Trait used for parsing files in different formats, such as JSON and plain text.

```rust
trait FileParser<T, +Serde<T>> {
    fn parse_txt(file: @File) -> Option<T>;
    fn parse_json(file: @File) -> Option<T>;
}
```

> ℹ️ **Info**
>
> Specific rules must be followed for snforge to correctly parse JSON and plain text files.
>
> Read more about them [here](../fs.md#file-format).

### Example for `parse_json`

File content:
```json
{{#include ../../../../listings/snforge_library_reference/data/user.json}}
```

Test code:
```rust
{{#include ../../../../listings/snforge_library_reference/tests/test_fs_parse_json.cairo}}
```

Let's run the test:

<!-- { "package_name": "snforge_library_reference" } -->
```shell
$ snforge test parse_json_example
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from snforge_library_reference package
Running 1 test(s) from tests/
User { age: 30, job: "Software Engineer", location: Location { city: "New York", country: "USA" }, name: "John", surname: "Doe" }
[PASS] snforge_library_reference_integrationtest::test_fs_parse_json::parse_json_example ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
