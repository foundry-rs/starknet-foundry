# Writing Tests

`snforge` lets you test standalone functions from your smart contracts. This technique is referred to as unit testing. You
should write as many unit tests as possible as these are faster than integration tests.

## Writing Your First Test

First, add the following code to the `src/lib.cairo` file:

```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/src/first_test.cairo}}
```

It is a common practice to keep your unit tests in the same file as the tested code.
Keep in mind that all tests in `src` folder have to be in a module annotated with `#[cfg(test)]`.
When it comes to integration tests, you can keep them in separate files in the `tests` directory.
You can find a detailed explanation of how `snforge` collects tests [here](test-collection.md).

Now run `snforge` using a command:

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from writing_tests package
Running 1 test(s) from src/
[PASS] writing::first_test::tests::test_sum
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

## Failing Tests

If your code panics, the test is considered failed. Here's an example of a failing test.

```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/src/panicking_tests.cairo:first_half}}
{{#include ../../listings/snforge_overview/crates/writing_tests/src/panicking_tests.cairo:second_half}}
```

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from writing_tests package
Running 1 test(s) from src/
[FAIL] writing_tests::panicking_tests::tests::failing

Failure data:
    0x616161 ('aaa')

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    writing_tests::panicking_tests::tests::failing
```
</details>
<br>

When contract fails, you can get backtrace information by setting the `SNFORGE_BACKTRACE=1` environment variable. Read more about it [here](../snforge-advanced-features/backtrace.md).

## Expected Failures

Sometimes you want to mark a test as expected to fail. This is useful when you want to verify that an action fails as
expected.

To mark a test as expected to fail, use the `#[should_panic]` attribute.

You can specify the expected failure message in three ways:

1. **With ByteArray**:
```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/tests/expected_failures.cairo:byte_array}}
```
With this format, the expected error message needs to be a substring of the actual error message. This is particularly useful when the error message includes dynamic data such as a hash or address.

2. **With felt**
```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/tests/expected_failures.cairo:felt}}
```

3. **With tuple of felts**:
```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/tests/expected_failures.cairo:tuple}}
```


```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from writing_tests package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] snforge_overview_integrationtest::should_panic_check_data
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```
</details>
<br>

## Ignoring Tests

Sometimes you may have tests that you want to exclude during most runs of `snforge test`.
You can achieve it using `#[ignore]` - tests marked with this attribute will be skipped by default.

```rust
{{#include ../../listings/snforge_overview/crates/writing_tests/tests/ignoring.cairo}}
```

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from writing_tests package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[IGNORE] writing_tests_integrationtest::ignoring::ignored_test
Tests: 0 passed, 0 failed, 0 skipped, 1 ignored, 0 filtered out
```
</details>
<br>

To run only tests marked with the  `#[ignore]` attribute use `snforge test --ignored`.
To run all tests regardless of the `#[ignore]` attribute use `snforge test --include-ignored`.

## Writing Assertions and `assert_macros` Package
> ⚠️ **Recommended only for development** ️⚠️
> 
>***Assert macros package provides a set of macros that can be used to write assertions such as `assert_eq!`.
In order to use it, your project must have the `assert_macros` dependency added to the `Scarb.toml` file.
These macros are very expensive to run on Starknet, as they result a huge amount of steps and are not recommended for production use. 
They are only meant to be used in tests.
For snforge `v0.31.0` and later, this dependency is added automatically when creating a project using `snforge init`. But for earlier versions, you need to add it manually.***

```toml
[dev-dependencies]
snforge_std = ...
assert_macros = "<scarb-version>"
```

Available assert macros are 
- `assert_eq!`
- `assert_ne!`
- `assert_lt!`
- `assert_le!`
- `assert_gt!`
- `assert_ge!`
