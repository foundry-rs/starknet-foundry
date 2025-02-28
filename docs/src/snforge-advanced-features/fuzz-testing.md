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

To convert a standard test into a random fuzz test, you need to add parameters to the test function
and include the [`#[fuzzer]`](../testing/test-attributes.md#fuzzer) attribute.
These arguments can then be used in the test body.
The test will be run many times against different randomly generated values.

```rust
{{#include ../../listings/fuzz_testing/src/basic_example.cairo}}
```

Then run `snforge test` like usual.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

<!-- TODO (#2926) -->
```shell
Collected 2 test(s) from fuzz_testing package
Running 2 test(s) from src/
[PASS] fuzz_testing::with_parameters::tests::test_sum (runs: 22, gas: {max: ~124, min: ~121, mean: ~123.00, std deviation: ~0.90})
[PASS] fuzz_testing::basic_example::tests::test_sum (runs: 256, gas: {max: ~124, min: ~121, mean: ~123.00, std deviation: ~0.81})
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 excluded, 0 filtered out
Fuzzer seed: [..]
```
</details>
<br>

## Types Supported by the Fuzzer

Fuzzer currently supports generating values for these types out of the box:

- `felt252`
- `u8`, `u16`, `u32`, `u64`, `u128`, `u256`
- `i8`, `i16`, `i32`, `i64`, `i128`
- `ByteArray`

To use other types, it is required to implement the [`Fuzzable`](../appendix/snforge-library/fuzzable.md) trait for them.
Providing non-fuzzable types will result in a compilation error.

## Fuzzer Configuration

It is possible to configure the number of runs of the random fuzzer as well as its seed for a specific test case:

```rust
{{#include ../../listings/fuzz_testing/src/with_parameters.cairo}}
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
