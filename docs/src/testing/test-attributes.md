## Test Attributes

`snforge` allows setting test attributes for test cases in order to modify their behavior.

Currently, those attributes are supported:

- `#[test]`
- `#[ignore]`
- `#[should_panic]`
- `#[available_gas]`
- `#[fork]`
- `#[fuzzer]`

### `#[test]`

Marks the function as a test case, to be visible for test collector.
Read more about test collection [here](./test-collection.md).

### `#[ignore]`

Marks the function as ignored, it will be skipped after collecting.
Use this if you don't want the test to be run (the runner will display how many tests were ignored in the summary).

Read more about the behavior and how to override
this [here](./testing.md#ignoring-some-tests-unless-specifically-requested).

### `#[should_panic]`

A test function can be marked with this attribute, in order to assert that the test function itself will panic.
If the test panics when marked with this attribute, it's considered as "passed".

Moreover, it can be used with either a tuple of shortstrings or a string for assessment of the exit panic data
(depending on what your contract throws).

#### Usage

Asserting the panic data can be done with multiple types of inputs:

ByteArray:

```rust
#[should_panic(expected: "No such file or directory (os error 2)")]
```

Shortstring:

```rust
#[should_panic(expected: 'panic message')]
```

Tuple of shortstrings:

```rust
#[should_panic(expected: ('panic message', 'second message', )]
```

Asserting that the function panics (any with any panic data):

```rust
#[should_panic]
```

### `#[available_gas]`

Sets a gas limit for the test.
If the test exceeds the limit, it fails with an appropriate error.

#### Usage

Asserts that the test does not use more than 5 units of gas.

```rust
#[available_gas(5)]
```

### `#[fork]`

Enables state forking for the given test case.

Read more about fork testing [here](../snforge-advanced-features/fork-testing.md).

#### Usage

Configures the fork endpoint with a given URL and a reference point for forking, which can be a block number, block
hash, or a named tag (only "latest" is supported).

| Reference Type | Example Usage                                                   |
|----------------|-----------------------------------------------------------------|
| `block_number` | `#[fork(url: "http://example.com", block_number: 123)]`         |
| `block_hash`   | `#[fork(url: "http://example.com", block_hash: 0x123deadbeef)]` |
| `block_tag`    | `#[fork(url: "http://example.com", block_tag: latest)]`         |

You can also define your frequently used fork configs in your `Scarb.toml`:

```toml
[[tool.snforge.fork]]
name = "TESTNET"
url = "http://your.rpc.url"
block_id.tag = "latest"
```

Then, instead of repeating them inside the attribute, you can reference them by the given name of the config declared
in `Scarb.toml`:

```rust
#[fork("TESTNET")] 
```

### `#[fuzzer]`

Enables fuzzing for a given test case.

Read more about test case fuzzing [here](../snforge-advanced-features/fuzz-testing.md).

#### Usage

Mark the test as fuzzed test, and configure the fuzzer itself.
Configures how many runs will be performed, and the starting seed (for repeatability).

```rust
#[fuzzer(runs: 10, seed: 123)]
```

Any parameter of `fuzzer` attribute can be omitted:

```rust
#[fuzzer]
#[fuzzer(runs: 10)]
#[fuzzer(seed: 123)]
```

And will be filled in with default values in that case (default `runs` value is 256).

> ⚠️ **Warning**
>
> Please note, that the test function needs to have some parameters in order for fuzzer to have something to fuzz.
> Otherwise it will fail to execute and crash the runner. 
