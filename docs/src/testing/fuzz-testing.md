# Fuzz Testing

In many cases, a test needs to verify function behavior for multiple possible values.
While it is possible to come up with these cases on your own, it is often impractical, especially when you want to test
against a large number of possible arguments.

> ℹ️**Info**
> Currently, Forge fuzzer only supports using randomly generated values.
> This way of fuzzing doesn't support any kind of value generation based on code analysis, test coverage or results of
> other fuzzer runs.
> In the future, more advanced fuzzing execution modes will be added.

## Random Fuzzing

To convert a test to a random fuzz test, simply add arguments to the test function.
These arguments can then be used in the test body.
The test will be run many times against different randomly generated values.

```rust
fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[test]
fn test_sum(x: felt252, y: felt252) {
    assert(sum(x, y) == x + y, 'sum incorrect');
}
```

Then run `snforge` like usual.

```shell
$ snforge
Collected 1 test(s) from package_name package
Running 1 test(s) from src/
[PASS] package_name::test_sum
Tests: 1 passed, 0 failed, 0 skipped
Fuzzer seed: [..]
```

> ⚠️ **Warning**
> Currently Forge only supports fuzz testing arguments with `felt252` type. Trying to use different argument types will
> result in an error.

## Fuzzer Configuration

It is possible to configure the number of runs of the random fuzzer as well as its seed too with command line arguments:

```shell
$ snforge --fuzzer-runs 1234 --fuzzer-seed 1111
```

Or in `Scarb.toml` file:

```toml
# ...
[tool.snforge]
fuzzer_runs = 1234
fuzzer_seed = 1111
# ...
```
