# `get_current_vm_step`

Gets the current step from Cairo VM during test execution.

```rust
fn get_current_vm_step() -> u32;
```

## Example

Contract code:

```rust
{{#include ../../../../listings/testing_reference/src/lib.cairo}}
```

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
Running 0 test(s) from src/
Running 1 test(s) from tests/
[FAIL] testing_reference_integrationtest::tests::test_setup_steps

Failure data:
    "assertion failed: `steps_end - steps_start <= 100`."

Tests: 0 passed, 1 failed, 0 ignored, 0 filtered out

Failures:
    testing_reference_integrationtest::tests::test_setup_steps
```
</details>
<br>

The test fails because the `setup` function exceeded the allowed number of steps (100 in this case).
