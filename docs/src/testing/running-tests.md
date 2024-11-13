# Running Tests

To run tests with `snforge`, simply run the `snforge test` command from the package directory.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 3 test(s) from package_name package
Running 3 test(s) from src/
[PASS] package_name::tests::executing
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
Tests: 3 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

## Filtering Tests

You can pass a filter string after the `snforge test` command to filter tests.
By default, any test with an [absolute module tree path](https://book.cairo-lang.org/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#paths-for-referring-to-an-item-in-the-module-tree)
 matching the filter will be run.

```shell
$ snforge test calling
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from package_name package
Running 2 test(s) from src/
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 1 filtered out
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
$ snforge test package_name::tests::calling --exact
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::tests::calling
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, other filtered out
```
</details>
<br>

## Stopping Test Execution After First Failed Test

To stop the test execution after first failed test, you can pass an `--exit-first` flag along with `snforge test` command.

```shell
$ snforge test --exit-first
```

<details>
<summary>Output:</summary>

```shell
Collected 6 test(s) from package_name package
Running 6 test(s) from src/
[PASS] package_name::tests::executing
[PASS] package_name::tests::calling
[PASS] package_name::tests::calling_another
[FAIL] package_name::tests::failing

Failure data:
    0x6661696c696e6720636865636b ('failing check')

Tests: 3 passed, 1 failed, 2 skipped, 0 ignored, 0 filtered out

Failures:
    package_name::tests::failing
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
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::tests::resources (gas: ~2213)
        steps: 881
        memory holes: 36
        builtins: ("range_check_builtin": 32)
        syscalls: (StorageWrite: 1, StorageRead: 1, CallContract: 1)

Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

For more information about how starknet-foundry calculates those, see [gas and resource estimation](gas-and-resource-estimation.md) section.