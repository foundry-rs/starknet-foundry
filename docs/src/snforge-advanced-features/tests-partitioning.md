# Tests Partitioning

When your test suite contains a large number of tests (especially fuzz tests), it can be helpful to split them into partitions and run each partition separately, for example in parallel CI jobs.


`snforge` supports this via the `--partition <INDEX/TOTAL>` flag.

When this flag is provided, `snforge` will divide all collected tests into `TOTAL` partitions and run only the partition with the given `INDEX` (1-based).

## Example

Let's consider package with the following 7 tests:

```rust
{{#include ../../listings/partitioning/tests/example.cairo}}
```

Running `snforge test --partition 1/2` will run tests `test_a`, `test_c`, `test_e`, `test_g` (4 tests), while running `snforge test --partition 2/2` will run tests `test_b`, `test_d`, `test_f` (3 tests).

<!-- { "package_name": "partitioning" } -->
```shell
$ snforge test --partition 1/2
```

<details>
<summary>Output:</summary>

```shell
Collected 4 test(s) from partitioning package
Running 4 test(s) from tests/
[PASS] partitioning_integrationtest::example::test_a ([..])
[PASS] partitioning_integrationtest::example::test_e ([..])
[PASS] partitioning_integrationtest::example::test_c ([..])
[PASS] partitioning_integrationtest::example::test_g ([..])
Running 0 test(s) from src/
Tests: 4 passed, 0 failed, 0 ignored, 0 filtered out

Finished partition run: 1/2
```

</details>


See example Github Actions workflow demonstrating partitioned test execution [here](../appendix/starknet-foundry-github-action.html#workflow-with-partitioned-tests).
