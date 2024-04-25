# Writing Tests

`snforge` lets you test standalone functions from your smart contracts. This technique is referred to as unit testing. You
should write as many unit tests as possible as these are faster than integration tests.

## Writing Your First Test

First, add the following code to the `src/lib.cairo` file:

```rust
fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[cfg(test)]
mod tests {
    use super::sum;

    #[test]
    fn test_sum() {
        assert(sum(2, 3) == 5, 'sum incorrect');
    }
}
```

It is a common practice to keep your unit tests in the same file as the tested code. 
Keep in mind that all tests in `src` folder have to be in a module annotated with `#[cfg(test)]`.
When it comes to integration tests, you can keep them in separate files in the `tests` directory.
You can find a detailed explanation of how `snforge` collects tests [here](test-collection.md).

Now run `snforge` using a command:

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::tests::test_sum
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

## Failing Tests

If your code panics, the test is considered failed. Here's an example of a failing test.

```rust
fn panicking_function() {
    let mut data = array![];
    data.append('aaa');
    panic(data)
}

#[cfg(test)]
mod tests {
    #[test]
    fn failing() {
        panicking_function();
        assert(2 == 2, '2 == 2');
    }
}
```

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[FAIL] package_name::tests::failing

Failure data:
    0x616161 ('aaa')

Tests: 0 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

Failures:
    package_name::tests::failing
```

## Expected Failures

Sometimes you want to mark a test as expected to fail. This is useful when you want to verify that an action fails as
expected.

To mark a test as expected to fail, use the `#[should_panic]` attribute. You can pass the expected failure message as an
argument to the attribute to verify that the test fails with the expected message
with `#[should_panic(expected: ('panic message', 'eventual second message',))]`.

```rust
#[test]
#[should_panic(expected: ('panic message', ))]
fn should_panic_check_data() {
    panic_with_felt252('panic message');
}
```

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::should_panic_check_data
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

## Ignoring Some Tests Unless Specifically Requested

Sometimes you may have tests that you want to exclude during most runs of `snforge test`.
You can achieve it using `#[ignore]` - tests marked with this attribute will be skipped by default.

```rust
#[test]
#[ignore]
fn ignored_test() {
    // test code
}
```

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[IGNORE] tests::ignored_test
Tests: 0 passed, 0 failed, 0 skipped, 1 ignored, 0 filtered out
```

To run only tests marked with the  `#[ignore]` attribute use `snforge test --ignored`. 
To run all tests regardless of the `#[ignore]` attribute use `snforge test --include-ignored`.

## Displaying Resources Used During Tests

To track resources like `builtins` / `syscalls` that are used when running tests, use `snforge test --detailed-resources`.

```shell
$ snforge test --detailed-resources
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::tests::resources (gas: ~2213)
        steps: 881
        memory holes: 36
        builtins: ("range_check_builtin": 32)
        syscalls: (StorageWrite: 1, StorageRead: 1, CallContract: 1)

Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
```

For more information about how starknet-foundry calculates those, see [gas and resource estimation](gas-and-resource-estimation.md) section.
