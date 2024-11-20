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
{{#include ../../listings/snforge_advanced_features/crates/fuzz_testing/src/basic_example.cairo}}
```

Then run `snforge test` like usual.

```shell
$ snforge test
```

<details>
<summary>Output:</summary>

```shell
Collected 2 test(s) from fuzz_testing package
Running 2 test(s) from src/
[PASS] fuzz_testing::with_parameters::tests::test_sum (runs: 22, gas: {max: ~1, min: ~1, mean: ~1.00, std deviation: ~0.00})
[PASS] fuzz_testing::basic_example::tests::test_sum (runs: 256, gas: {max: ~1, min: ~1, mean: ~1.00, std deviation: ~0.00})
Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
Fuzzer seed: [..]
```
</details>
<br>

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
{{#include ../../listings/snforge_advanced_features/crates/fuzz_testing/src/with_parameters.cairo}}
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
