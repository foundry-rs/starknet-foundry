# The Manifest Format

The `Scarb.toml` contains the package manifest that is needed in package compilation process. It can be used to provide configuration for Starknet Foundry Forge. For more, see [official Scarb documentation](https://docs.swmansion.com/scarb/docs/reference/manifest.html).

## `Scarb.toml` Contents

### `[tool.snforge]`
```toml
[tool.snforge]
# ...
```
Allows to configure `snforge` settings. All fields are optional.

#### `exit_first`
The `exit_first` fields specifies whether to stop tests execution immediately upon the first failure. See more about [stopping test execution after first failed test](https://foundry-rs.github.io/starknet-foundry/testing/running-tests.html#stopping-test-execution-after-first-failed-test).

```toml
[tool.snforge]
exit_first = true
```

#### `fuzzer_runs`
The `fuzzer_runs` field specifies the number of runs of the random fuzzer. 

#### `fuzzer_seed`
The `fuzzer_seed` field specifies the seed for the random fuzzer.

See more about [fuzzer](https://foundry-rs.github.io/starknet-foundry/testing/test-attributes.html#fuzzer).

#### Example of fuzzer configuration

```toml
[tool.snforge]
fuzzer_runs = 1234
fuzzer_seed = 1111
```

### `[[tool.snforge.fork]]`
```toml
[[tool.snforge.fork]]
# ...
```
Allows to configure forked tests. If defined, all fields outlined below must also be defined. See more about [fork testing](https://foundry-rs.github.io/starknet-foundry/testing/test-attributes.html#fork).

#### `name`
The `name` field specifies the name of the fork.
```toml
[[tool.snforge.fork]]
name = "SOME_NAME"
```

#### `url`
The `url` field specifies the address of RPC provider.
```toml
[[tool.snforge.fork]]
url = "http://your.rpc.url"
```

#### `block_id.<tag|number|hash>`
The `block_id` field specifies the block to fork from. It can be specified by `tag`, `number` or `hash`.

```toml
[[tool.snforge.fork]]
block_id.hash = "0x123"
```

#### Example configuration with two forks

```toml
[[tool.snforge.fork]]
name = "SOME_NAME"
url = "http://your.rpc.url"
block_id.tag = "latest"

[[tool.snforge.fork]]
name = "SOME_SECOND_NAME"
url = "http://your.second.rpc.url"
block_id.number = "123"
```

### `[tool.scarb]`

```toml
[tool.scarb]
allow-prebuilt-plugins = ["snforge_std"]
```
Note: This configuration requires Scarb version >= 2.10.0 .

It allows `scarb` to download precompiled dependencies used by `snforge_std` from [the registry](https://scarbs.xyz).
The `snforge_std` library depends on a Cairo plugin that is written in Rust, and otherwise is compiled locally on the user's side.

### `[profile.<dev|release>.cairo]`
By default, these arguments do not need to be defined. Only set them to use [profiler](https://foundry-rs.github.io/starknet-foundry/snforge-advanced-features/profiling.html#profiling) or [coverage](https://foundry-rs.github.io/starknet-foundry/testing/coverage.html#coverage).

Adjust Cairo compiler configuration parameters when compiling this package. These options are not taken into consideration when this package is used as a dependency for another package. All fields are optional.

```toml
[profile.dev.cairo]
# ...
```

#### `unstable-add-statements-code-locations-debug-info`
See [`unstable-add-statements-code-locations-debug-info`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#unstable-add-statements-code-locations-debug-info) in Scarb documentation.

```toml
[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
```

#### `unstable-add-statements-functions-debug-info`
See [`unstable-add-statements-functions-debug-info`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#unstable-add-statements-functions-debug-info) in Scarb documentation.
```toml
[profile.dev.cairo]
unstable-add-statements-functions-debug-info = true
```

#### `inlining-strategy`
See [`inlining-strategy`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#inlining-strategy) in Scarb documentation.
```toml
[profile.dev.cairo]
inlining-strategy = "avoid"
```

#### Example of configuration which allows [coverage](https://foundry-rs.github.io/starknet-foundry/testing/coverage.html) report generation
```toml
[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
unstable-add-statements-functions-debug-info = true
inlining-strategy = "avoid"
```

### `[features]`
A package defines a set of named features in the `[features]` section of `Scarb.toml` file. Each defined feature can list other features that should be enabled with it. All fields are optional.
```toml
[features]
# ...
```

#### `<feature-name>`
The `<feature-name>` field specifies the name of the feature and list of other features that should be enabled with it.
See [features](https://docs.swmansion.com/scarb/docs/reference/conditional-compilation.html#features) in Scarb documentation.
```toml
[features]
enable_for_tests = []
```

#### Example of `Scarb.toml` allowing conditional contracts compilation
Firstly, define a contract in the src directory with a `#[cfg(feature: '<FEATURE_NAME>')]` attribute:
```rust
#[starknet::contract]
#[cfg(feature: 'enable_for_tests')]
mod MockContract {
    // ...
}
```

Then update Scarb.toml so it includes the following lines:
```toml
[features]
enable_for_tests = []
```

### `[[target.starknet-contract]]`
The `starknet-contract` target allows to build the package as a Starknet Contract. See more about [Starknet Contract Target](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#starknet-contract-target) in Scarb documentation.

```toml
[[target.starknet-contract]]
# ...
```

#### `sierra`
See more about [Sierra contract class generation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#sierra-contract-class-generation) in Scarb documentation.
```toml
[[target.starknet-contract]]
sierra = true
```

#### `casm`

Enabling `casm = true` in Scarb.toml causes unnecessary overhead and should be disabled unless required by other tools. Tools like `snforge` and `sncast` recompile Sierra to CASM separately, resulting in redundant processing. This duplicates CASM generation, significantly impacting performance, especially for large Sierra programs. See more about [CASM contract class generation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#casm-contract-class-generation) in Scarb documentation.


```toml
[[target.starknet-contract]]
casm = true
```

#### `build-external-contracts`
The `build-external-contracts` allows to use contracts from your dependencies inside your tests. It accepts a list of strings, each of which is a reference to a contract defined in a dependency. You need to add dependency which implements this contracts to your Scarb.toml. See more about [compiling external contracts](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#compiling-external-contracts) in Scarb documentation. 

```toml
[[target.starknet-contract]]
build-external-contracts = ["openzeppelin::account::account::Account"]
```

#### Example of configuration which allows to use external contracts in tests
```toml
# ...
[dependencies]
starknet = ">=2.8.2"
openzeppelin = { git = "https://github.com/OpenZeppelin/cairo-contracts.git", branch = "cairo-2" }

[[target.starknet-contract]]
build-external-contracts = ["openzeppelin::account::account::Account"]
# ...
```

#### Complete example of `Scarb.toml`
```toml
[package]
name = "example_package"
version = "0.1.0"
edition = "2023_11"

# See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

[dependencies]
starknet = "2.8.2"

[dev-dependencies]
snforge_std = "{{snforge_std_version}}"
starknet = ">=2.8.2"
openzeppelin = { git = "https://github.com/OpenZeppelin/cairo-contracts.git", branch = "cairo-2" }

[[target.starknet-contract]]
sierra = true
build-external-contracts = ["openzeppelin::account::account::Account"]

[scripts]
test = "snforge test"
# foo = { path = "vendor/foo" }

[tool.snforge]
exit_first = true
fuzzer_runs = 1234
fuzzer_seed = 1111

[[tool.snforge.fork]]
name = "SOME_NAME"
url = "http://your.rpc.url"
block_id.tag = "latest"

[[tool.snforge.fork]]
name = "SOME_SECOND_NAME"
url = "http://your.second.rpc.url"
block_id.number = "123"

[[tool.snforge.fork]]
name = "SOME_THIRD_NAME"
url = "http://your.third.rpc.url"
block_id.hash = "0x123"

[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
unstable-add-statements-functions-debug-info = true
inlining-strategy = "avoid"

[features]
enable_for_tests = []
```
