# Writing Tests

Forge lets you test standalone functions from your smart contracts. This technique is referred to as unit testing. You
should write as many unit tests as possible as these are faster than integration tests.

## Writing your first test

First, add the following code to the `src/lib.cairo` file:

```rust
fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[test]
fn test_sum() {
    assert(sum(2, 3) == 5, 'sum incorrect');
}
```

It is a common practice to keep your unit tests in the same file as the tested code. 
When it comes to integration tests, you can keep them in separate files in the `tests` directory.
You can find a detailed explanation of how Forge collects tests [here](test-collection.md).

Now run forge using a command:

```shell
$ snforge
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::test_sum
Tests: 1 passed, 0 failed, 0 skipped
```

## Failing tests

If your code panics, the test is considered failed. Here's an example of a failing test.

```rust
use array::ArrayTrait;

fn panicking_function() {
    let mut data = ArrayTrait::new();
    data.append('aaa');
    panic(data)
}

#[test]
fn failing() {
    panicking_function();
    assert(2 == 2, '2 == 2');
}
```

```shell
$ snforge
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[FAIL] package_name::failing

Failure data:
    [6381921], converted to a string: [aaa]

Tests: 0 passed, 1 failed, 0 skipped

Failures:
    package_name::failing
```

## Expected failures

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
$ snforge
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] src::should_panic_check_data
Tests: 1 passed, 0 failed, 0 skipped
```
