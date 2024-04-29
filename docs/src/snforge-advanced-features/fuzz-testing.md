# Fuzz Testing

In many cases, a test needs to verify function behavior for multiple possible values.
While it is possible to come up with these cases on your own, it is often impractical, especially when you want to test
against a large number of possible arguments.

> ℹ️ **Info**
> Currently, `snforge` fuzzer only supports using randomly generated values.
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

Then run `snforge test` like usual.

```shell
$ snforge test
Collected 1 test(s) from package_name package
Running 0 test(s) from src/
Running 1 test(s) from tests/
[PASS] tests::test_sum (runs: 256)
Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
Fuzzer seed: [..]
```

## Types Supported by the Fuzzer

Fuzzer currently supports generating values of these types

- `u8`
- `u16`
- `u32`
- `u64`
- `u128`
- `u256`
- `felt252`

Trying to use arguments of different type in test definition will result in an error.

## Fuzzer Configuration

It is possible to configure the number of runs of the random fuzzer as well as its seed for a specific test case:

```rust
#[test]
#[fuzzer(runs: 22, seed: 38)]
fn test_sum(x: felt252, y: felt252) {
    assert(sum(x, y) == x + y, 'sum incorrect');
}
```

It can also be configured globally, via command line arguments:

```shell
$ snforge test --fuzzer-runs 1234 --fuzzer-seed 1111
```

Or in `Scarb.toml` file:

```toml
# ...
[tool.snforge]
fuzzer_runs = 1234
fuzzer_seed = 1111
# ...
```
