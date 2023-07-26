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

It is a common practice to keep your unit tests in the same file as the tested code. If you prefer, you can also put
test code in a separate file anywhere in the project directory.

Now run forge using a command:

```shell
$ snforge
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[PASS] src::test_sum
Tests: 1 passed, 0 failed, 0 skipped
```

## Test collecting

Forge considers all functions in your project marked with `#[test]` attribute as tests.
Test functions cannot return any values and cannot take any arguments.

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
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from src/lib.cairo
[FAIL] src::failing

Failure data:
    [6381921], converted to a string: [aaa]

Tests: 0 passed, 1 failed, 0 skipped
```
