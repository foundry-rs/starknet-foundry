# Fuzz Testing

In many cases, a test needs to verify function behavior for multiple possible values.
While it is possible to come up with this cases on your own, it is often impractical, especially when you want to test
for a large number of possible arguments.

Forge allows easily converting your tests to fuzz tests, so random values are generated automatically and injected into
a test.

> ⚠️ **Warning**
> Currently Forge only supports fuzz testing arguments with `felt252` type. Trying to use different argument types will
> result in an error.

## Converting Tests To Fuzz Tests

To convert a test to a fuzz test, simply add arguments to the test function.
These arguments can then be used in the test body.
The test will be run multiple times for multiple randomly generated values.

```cairo
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
Collected 1 test(s) and 1 test file(s)
Running 1 test(s) from package_name package
Running fuzzer for package_name::test_sum, 256 runs:
[PASS] package_name::test_sum
```

## Configuring The Fuzzer

It is possible to configure the number of runs of the fuzzer as well as its seed either by a command line argument:

```shell
$ snforge --fuzzer-runs 1234 --fuzzer-seed 1111
```

Or in `Scarb.toml` file:

```toml
# ...
[tool.snforge]
fuzzer_runs = 1234
fuzzer_seed = 1111
```
