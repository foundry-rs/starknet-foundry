# Running Tests

To run tests with `snforge`, simply run the `snforge test` command from the package directory.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from hello_snforge package
Running 0 test(s) from src/
Running 3 test(s) from tests/
[PASS] hello_snforge_integrationtest::test_contract::test_calling (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
[PASS] hello_snforge_integrationtest::test_contract::test_executing (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
[PASS] hello_snforge_integrationtest::test_contract::test_calling_another (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
Tests: 3 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
```
</details>
<br>

## Filtering Tests

You can pass a filter string after the `snforge test` command to filter tests.
By default, any test with an [absolute module tree path](https://book.cairo-lang.org/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree) matching the filter will be run.

```shell
$ snforge test calling
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from hello_snforge package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] hello_snforge_integrationtest::test_contract::test_calling_another (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
[PASS] hello_snforge_integrationtest::test_contract::test_calling (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 1 filtered out
```
</details>
<br>

## Running a Specific Test

To run a specific test, you can pass a filter string along with an `--exact` flag.
Note, you have to use a fully qualified test name, including a module name.

> 📝 **Note**
>
> Running a specific test results in optimized compilation. `snforge` will try to compile only the desired test, unlike the case of running all tests where all of them are compiled.
>

```shell
$ snforge test hello_snforge_integrationtest::test_contract::test_calling --exact
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from hello_snforge package
Running 1 test(s) from tests/
[PASS] hello_snforge_integrationtest::test_contract::test_calling (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~40000)
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, other filtered out
```
</details>
<br>

## Excluding Tests

You can use the `--exclude` flag to run all tests *except* those matching a specified filter pattern. This is useful for temporarily excluding problematic tests or focusing on a subset of tests by excluding others.

```shell
$ snforge test --exclude "failing"
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from hello_snforge package
Running 0 test(s) from src/
Running 2 test(s) from tests/
[PASS] hello_snforge_integrationtest::test_contract::test_calling (gas: ~1)
[PASS] hello_snforge_integrationtest::test_contract::test_executing (gas: ~1)
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 1 excluded, 0 filtered out
```
</details>
<br>

The excluded tests are still counted in the test summary but are not executed.

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `snforge test` command.

<!-- { "ignored": true } -->
```shell
$ snforge test --exit-first
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from failing_example package
Running 3 test(s) from tests/
[FAIL] failing_example_tests::test_failing

Failure data:
    0x6661696c696e6720636865636b ('failing check')


Failures:
    failing_example_tests::test_failing

Tests: 0 passed, 1 failed, 2 skipped, 0 ignored, 0 excluded, 0 filtered out
```
</details>
<br>

## Displaying Resources Used During Tests

To track resources like `builtins` / `syscalls` that are used when running tests, use `snforge test --detailed-resources`.

```shell
$ snforge test --detailed-resources
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from hello_starknet package
Running 2 test(s) from tests/
[PASS] hello_starknet_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (l1_gas: ~0, l1_data_gas: ~96, l2_gas: ~360000)
        steps: 3405
        memory holes: 22
        builtins: (range_check: 77, pedersen: 7)
        syscalls: (CallContract: 2, StorageRead: 1, Deploy: 1)

[PASS] hello_starknet_integrationtest::test_contract::test_increase_balance (l1_gas: ~0, l1_data_gas: ~192, l2_gas: ~480000)
        steps: 4535
        memory holes: 15
        builtins: (range_check: 95, pedersen: 7)
        syscalls: (CallContract: 3, StorageRead: 3, Deploy: 1, StorageWrite: 1)

Running 0 test(s) from src/
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
```
</details>
<br>

For more information about how starknet-foundry calculates those, see [gas and resource estimation](gas-and-resource-estimation.md) section.