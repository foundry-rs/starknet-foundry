# `get_current_vm_step`

Gets the current step during test execution.

```rust
fn get_current_vm_step() -> u32;
```

## Example

Test code:

```rust
{{#include ../../../../listings/testing_reference/tests/tests.cairo}}
```

<!-- { "package_name": "testing_reference" } -->
Let's run the test:
```shell
$ snforge test test_setup_steps
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from testing_reference package
Running 1 test(s) from tests/
[PASS] testing_reference_integrationtest::tests::test_setup_steps ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out
```
</details>
<br>
