# The Manifest Format

The `Scarb.toml` contains the package manifest that is needed in package compilation process. It can be used to provide configuration for Starknet Foundry Forge. Official docs can be found [here](https://doc.rust-lang.org/cargo/reference/manifest.html).

## `Scarb.toml` Contents

### `[tool.snforge]`
```toml
[tool.snforge]
# ...
```
Allows to configure `snforge` settings. All fields are optional.

#### `exit_first`
The `exit_first` fields specifies whether to stop tests execution immediately upon the first failure.
```toml
[tool.snforge]
exit_first = true
```
See more about [stopping test execution after first failed test](https://foundry-rs.github.io/starknet-foundry/testing/running-tests.html#stopping-test-execution-after-first-failed-test).

#### `fuzzer_runs`
The `fuzzer_runs` field specifies the number of runs of the random fuzzer.

#### `fuzzer_seed`
The `fuzzer_seed` field specifies the seed for the random fuzzer.

#### Example of fuzzer configuration
```toml
[tool.snforge]
fuzzer_runs = 1234
fuzzer_seed = 1111
```
See more about [fuzzer](https://foundry-rs.github.io/starknet-foundry/testing/test-attributes.html#fuzzer).

### `[[tool.snforge.fork]]`
```toml
[[tool.snforge.fork]]
# ...
```
Allows to configure forked tests. While in use, all below fields must be set at the same time.

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
See more about [fork testing](https://foundry-rs.github.io/starknet-foundry/testing/test-attributes.html#fork).

### `[profile.<dev|release>.cairo]`
Adjust Cairo compiler configuration parameters when compiling this package. These options are not taken into consideration when this package is used as a dependency for another package. All fields are optional.

```toml
[profile.dev.cairo]
# ...
```

#### `unstable-add-statements-code-locations-debug-info`
See [`unstable-add-statements-code-locations-debug-info`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#unstable-add-statements-code-locations-debug-info) in Scarb docs.

```toml
[profile.dev.cairo]
unstable-add-statements-code-locations-debug-info = true
```

#### `unstable-add-statements-functions-debug-info`
See [`unstable-add-statements-functions-debug-info`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#unstable-add-statements-functions-debug-info) in Scarb docs.
```toml
[profile.dev.cairo]
unstable-add-statements-functions-debug-info = true
```

#### `inlining-strategy`
See [`inlining-strategy`](https://docs.swmansion.com/scarb/docs/reference/manifest.html#inlining-strategy) in Scarb docs.
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

See [features](https://docs.swmansion.com/scarb/docs/reference/conditional-compilation.html#features) in Scarb docs.

### `[[target.starknet-contract]]`
Scarb supports registering targets that are handled by Scarb extensions. Such targets are called external. All fields are optional.
```toml
[[target.starknet-contract]]
# ...
```

#### `sierra`
The `sierra` fields specifies whether Sierra codegen should be enabled.
```toml
[[target.starknet-contract]]
sierra = true
```

#### `casm`
The `casm` fields specifies whether Casm codegen should be enabled.
```toml
[[target.starknet-contract]]
casm = true
```

See more about [targets](https://docs.swmansion.com/scarb/docs/reference/targets.html) in scarb docs.


#### `build-external-contracts`
The `build-external-contracts` accepts a list of strings, each of which is a reference to a contract defined in a dependency. The package that implements this contracts need to be declared as a dependency of the project in `[dependencies]`.
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
snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry", tag = "v0.30.0" }
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
