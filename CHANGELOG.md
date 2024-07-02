# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Cast

- added support for v3 transactions on account deploy

## [0.25.0] - 2024-06-12

### Forge

#### Changed

- `SyscallResultStringErrorTrait::map_error_to_string` removed in favor of utility function (`snforge_std::byte_array::try_deserialize_bytearray_error`)
- Updated event testing - read more [here](./docs/src/testing/testing-events.md) on how it now works and [here](./docs/src/appendix/cheatcodes/spy_events.md)
about updated `spy_events` cheatcode

### Cast

#### Removed
- `--class-hash` flag from `account deploy` command

#### Added

- `tx-status` subcommand to get transaction status. [Read more here](./docs/src/starknet/tx-status.md)
- `tx_status` function to cast_std. [Read more here](./docs/src/appendix/sncast-library/tx_status.md)
- Support for creating argent accounts
- Support for creating braavos accounts

## [0.24.0] - 2024-05-22

### Forge

#### Removed

- `prank`, `warp`, `roll`, `elect`, `spoof` cheatcodes in favour of `cheat_execution_info`

#### Added

- `cheat_execution_info` cheatcode and per variable helpers for it

### Cast


#### Added

- New required flag `--type` to `account add` command

### Forge

#### Changed

- `SignerTrait::sign` now returns `Result` instead of failing the test

- `L1HandlerTrait::execute()` takes source address and payloads as arguments [Read more here](https://foundry-rs.github.io/starknet-foundry/appendix/cheatcodes/l1_handler.html)

- When calling to an address which does not exists, error is forwarded to cairo runtime instead of failing the test

## [0.23.0] - 2024-05-08

### Forge

#### Removed
- `event_name_hash` removal, in favour of `selector!` usage

#### Changed

- the tool now always compiles Sierra contract artifacts to CASM using
[`USC`](https://github.com/software-mansion/universal-sierra-compiler) - before it used to consume CASM artifacts
produced by Scarb if they were present. Setting up `casm = true` in `Scarb.toml` is no longer recommended - it may slow
down the compilation.
- The `replace_bytecode` cheatcode now returns `Result` with a possible `ReplaceBytecodeError`, since it may cause unexpected errors down the line when not handled properly

### Cast

#### Changed

- the tool now always compiles Sierra contract artifacts to CASM using
[`USC`](https://github.com/software-mansion/universal-sierra-compiler) - before it used to consume CASM artifacts
produced by Scarb if they were present. Setting up `casm = true` in `Scarb.toml` is no longer recommended - it may slow
down the compilation.

#### Fixed
- scripts built with release profile are now properly recognized and ran 

## [0.22.0] - 2024-04-17

### Forge

#### Changed

- `deploy` / `deploy_at` now additionally return the constructor return data via `SyscallResult<(ContractAddress, Span<felt252>)>`
- `declare` returns `Result<ContractClass, Array<felt252>>` instead of `ContractClass`
- `L1HandlerTrait::execute()` returns `SyscallResult<()>`
- `SyscallResultStringErrorTrait::map_string_error` renamed to `SyscallResultStringErrorTrait::map_error_to_string`
- `var` now supports `ByteArray` with double quoting, and returns `Array<felt252>` instead of a single `felt252`

#### Removed
- `snforge_std::RevertedTransaction`

## [0.21.0] - 2024-04-03

### Forge

#### Changed

- `read_txt` and `read_json` now supports `ByteArray`

### Cast

#### Added

- sncast script idempotency feature - every action done by the script that alters the network state will be tracked in state file, 
and won't be replayed if previously succeeded 

## [0.20.1] - 2024-03-22

## [0.20.0] - 2024-03-20

### Forge

#### Added

- variants of cheatcodes with `CheatSpan` (read more [here](https://foundry-rs.github.io/starknet-foundry/testing/using-cheatcodes#setting-cheatcode-span))
- Providing configuration data with env variables [DOCS](https://foundry-rs.github.io/starknet-foundry/projects/configuration.html#environmental-variables)

#### Fixed

- Events emitted in cairo 0 contracts are now properly collected
- `--build-profile` no longer fails silently (compatible with [`cairo-profiler`](https://github.com/software-mansion/cairo-profiler) 0.2.0)

#### Changed

- Default `chain_id` has been changed from `SN_GOERLI` to `SN_SEPOLIA`
- Supported RPC version is now 0.7.0
- Gas calculation is in sync with starknet 0.13.1 (with EIP 4844 blob usage enabled)
- Resources displayed (steps, builtins) now include OS costs of syscalls 

### Cast

#### Added

- Support for OpenZeppelin Cairo 1 (or higher) accounts creation, deployment and usage
- Providing configuration data with env variables [DOCS](https://foundry-rs.github.io/starknet-foundry/projects/configuration.html#environmental-variables)

#### Changed

- Supported RPC version is now 0.7.0
- Default class hash in `account create` and `account deploy` has been changed to [cairo2 class hash](https://starkscan.co/class/0x04c6d6cf894f8bc96bb9c525e6853e5483177841f7388f74a46cfda6f028c755)

## [0.19.0] - 2024-03-06

### Forge

⚠️ This version requires installing external [universal-sierra-compiler (v2.0.0)](https://github.com/software-mansion/universal-sierra-compiler) ⚠️

#### Added

- [`replace_bytecode`](https://foundry-rs.github.io/starknet-foundry/appendix/cheatcodes/replace_bytecode.html) cheatcode
- result of the call to the trace
- added `--build-profile` flag to the `--test` command. Saves trace data and then builds profiles of test cases which pass and are not fuzz tests. You need [cairo-profiler](https://github.com/software-mansion/cairo-profiler) installed on your system.
- dependency on the [universal-sierra-compiler](https://github.com/software-mansion/universal-sierra-compiler)
binary, which will allow forge to be independent of sierra version


#### Changed

- `var()`, `read_txt()`, `read_json()`, `FileTrait::new()`, `declare()` now use regular strings (`ByteArray`) instead of short strings (`felt252`)
- `start_mock_call()`, `stop_mock_call()`, `L1Handler` now use selector (`selector!()`) instead of names

### Cast

#### Changed

- `declare()` now uses regular strings (`ByteArray`) instead of short strings (`felt252`)
- `call()` and `invoke()` now require function selector (`selector!()`) instead of function name in scripts (sncast_std)

#### Removed

- `--path-to-scarb-toml` optional flag that allowed to specify the path to the `Scarb.toml` file
- `--deployed` flag from `account add` subcommand

## [0.18.0] - 2024-02-21

### Forge

#### Added

- contract names to call trace
- `--max-n-steps` argument that allows setting own steps limit

#### Changed

- Unknown entry point error when calling a contract counts as a panic
- Cairo edition set to `2023_11`

#### Fixed

- Calling Cairo 0 contract no longer cancels cheatcodes in further calls

### Cast

#### Added

- `script init` command to generate a template file structure for deployment scripts
- Warning is emitted when executing sncast commands if the node's JSON-RPC version is incompatible

#### Changed

- to run a deployment script it is required to use `script run` subcommand

## [0.17.1] - 2024-02-12

### Cast

#### Changed

- fixed a bug where a profile was passed to scarb even when it did not exist
- error handling from inside deployment scripts is now possible (`declare`, `deploy`, `call`, `invoke` now return `Result<T, ScriptCommandError>`)

### Forge

#### Added

- `map_string_error` for use with dispatchers, which automatically converts string errors from the syscall result (read more [here](https://foundry-rs.github.io/starknet-foundry/testing/contracts#handling-errors))

## [0.17.0] - 2024-02-07

### Forge

#### Added

- Warning in fork testing is emitted, when node JSON-RPC version is incompatible
- `get_call_trace` library function for retrieving call trace in tests

#### Changed

- Gas estimation is now aligned with the Starknet v0.13

#### Removed

- `snforge_std::PrintTrait` - use `print!`, `println!` macros and / or `core::debug::PrintTrait` instead

#### Fixed

- Gas used in constructors and handling of L1 messages is now properly included in total gas cost

### Cast

#### Changed

- sncast tool configuration is now moved away from `Scarb.toml` to `snfoundry.toml` file. This file must be present in current or any parent directories in order to use profiles.

#### Added

- `--package` flag for `declare` and `script` subcommands, that specifies scarb package to work with
- `Debug` and `Display` impls for script subcommand responses - use `print!`, `println!` macros instead of calling `.print()`

## [0.16.0] - 2024-01-26

### Forge

#### Added
- Bump to cairo 2.5.0

#### Changed

- `SafeDispatcher`s usages need to be tagged with `#[feature("safe_dispatcher)]` (directly before usage), see [the shamans post](https://community.starknet.io/t/cairo-v2-5-0-is-out/112807#safe-dispatchers-15)

## [0.15.0] - 2024-01-24

### Forge

#### Added

- `--detailed-resources` flag for displaying additional info about used resources
- `store` and `load` cheatcodes
- `--save-trace-data` flag to `snforge test` command. Traces can be used for profiling purposes.

#### Changed

- `available_gas` attribute is now supported (Scarb >= 2.4.4 is required)

#### Fixed

- Error message for tests that should panic but pass

### Cast

#### Changed

- the 'pending' block is used instead of 'latest' as the default when obtaining the nonce

## [0.14.0] - 2024-01-11

### Forge

#### Added

- `Secp256k1` and `Secp256r1` curves support for `KeyPair` in `snforge_std`

#### Changed

- maximum number of computational steps per call set to current Starknet limit (3M)
- `mean` and `std deviation` fields are displayed for gas usage while running fuzzing tests 
- Cairo edition in `snforge_std` and `sncast_std` set to `2023_10`
- `snforge_std::signature` module with `stark_curve`, `secp256k1_curve` and `secp256r1_curve` submodules

#### Fixed

- Safe library dispatchers in test code no longer propagate errors when not intended to

## [0.13.1] - 2023-12-20

### Forge

#### Added

- `assert_not_emitted` assert to check if an event was not emitted

#### Changed 

- fields from `starknet::info::v2::TxInfo` are now part of `TxInfoMock` from `snforge_std::cheatcodes::tx_info`
- consistent latest block numbers for each url are now used across the whole run when testing against forks

#### Fixed

- Parsing panic data from call contract result

### Cast

#### Added 

- add support for sepolia network
- `--yes` option to `account delete` command that allows to skip confirmation prompt

#### Changed

- Argument `max-fee` in `account deploy` is now optional

## [0.13.0] - 2023-12-14

### Forge

#### Changed

- Bump cairo to 2.4.0.
- Migrated test compilation and collection to Scarb, snforge should now be compatible with every Scarb version >= 2.4.0 unless breaking changes happen

## [0.12.0] - 2023-12-06

### Forge

#### Added

- print gas usage for each test
- Support for test collector built-in in Scarb with the `--use-scarb-collector` flag. Requires at least `nightly-2023-12-04` version of Scarb.

### Cast

#### Added

- `--wait-timeout` to set timeout for waiting for tx on network using `--wait` flag (default 60s)
- `--wait-retry-interval` to adjust the time between consecutive attempts to fetch tx from network using `--wait` flag (default 5s)
- allow setting nonce in declare, deploy and invoke (using `--nonce` and in deployment scripts)
- add `get_nonce` function to cast_std
- `--private-key-file` option to `account add` command that allows to provide a path to the file holding account private key

## [0.11.0] - 2023-11-22

### Forge

#### Added

- `elect` cheatcode for mocking the sequencer address. Read more [here](./docs/src/appendix/cheatcodes/sequencer_address/start_elect.md).
- `--rerun-failed` option to run tests that failed during the last run.

#### Changed
- `start_warp` and `stop_warp` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/block_timestamp/start_warp.md). 
- `start_prank` and `stop_prank` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/caller_address/start_prank.md).
- `start_roll` and `stop_roll` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/block_number/start_roll.md).

PS: Credits to @bllu404 for the help with the new interfaces for cheats!

#### Fixed

- using unsupported `available_gas` attribute now fails the specific test case instead of the whole runner

### Cast

### Added

- MVP for cairo deployment scripts with declare, deploy, invoke and call

## [0.10.2] - 2023-11-13

### Forge

#### Changed

- Bump cairo to 2.3.1

#### Removed

- `available_gas` attribute, it didn't compute correctly gas usage. Contract functions execution cost would not be included.

## [0.10.1] - 2023-11-09

### Cast

#### Fixed

- scarb metadata in declare subcommand now takes manifest path from cli if passed instead of looking for it

## [0.10.0] - 2023-11-08

### Forge

#### Removed

- forking of the `Pending` block

#### Added

- `--color` option to control when colored output is used
- when specifying `BlockId::Tag(Latest)` block number of the used block will be printed
- printing number of ignored and filtered out tests

#### Fixed

- Segment Arena Builtin crashing with `CairoResourcesNotContainedInFeeCosts` when Felt252Dict was used

### Cast

#### Fixed

- account commands now always return valid json when `--json` flag is passed
- allow passing multiple calldata argument items without quotes
- display correct error message when account file is invalid

## [0.9.1] - 2023-10-30

### Forge

#### Fixed

- diagnostic paths referring to `tests` folder
- caching `get_class_hash_at` in forking test mode (credits to @jainkunal for catching the bug)

## [0.9.0] - 2023-10-25

### Forge

#### Added

- `#[ignore]` attribute together with `--ignored` and `include-ignored` flags - read more [here](https://foundry-rs.github.io/starknet-foundry/testing/testing.html#ignoring-some-tests-unless-specifically-requested)
- support for `deploy_syscall` directly in the test code (alternative to `deploy`)
- `snforge_std::signature` module for performing ecdsa signatures

#### Changed

- updated Cairo version to 2.3.0 - compatible Scarb version is 2.3.0:
  - tests in `src` folder now have to be in a module annotated with `#[cfg(test)]`
- `snforge_std::PrintTrait` will not convert values representing ASCII control characters to strings
- separated `snforge` to subcommands: `snforge test`, `snforge init` and `snforge clean-cache`. 
Read more [here](https://foundry-rs.github.io/starknet-foundry/appendix/snforge.html).
- `starknet::get_block_info` now returns correct block info in a forked block

### Cast

#### Added

- `show-config` subcommand to display currently used configuration
- `account delete` command for removing accounts from the accounts file
- `--hex-format` flag has been added

#### Removed
- `-i` short for `--int-format` is removed, now have to use the full form `--int-format`

## [0.8.3] - 2023-10-17

### Forge 

#### Changed

- Test from different crates are no longer run in parallel
- Test outputs are printed in non-deterministic order

#### Fixed

- Test output are printed in real time again
- Bug when application would not wait for tasks to terminate after execution was cancelled

## [0.8.2] - 2023-10-12

### Forge

#### Fixed
- incorrect caller address bug

## [0.8.1] - 2023-10-12
### Forge

#### Fixed
- significantly reduced ram usage

## [0.8.0] - 2023-10-11

### Forge

#### Added

- `#[fuzzer(...)]` attribute allowing to specify a fuzzer configuration for a single test case
- Support for `u8`, `u16`, `u32`, `u64`, `u128`, `u256` types to fuzzer
- `--clean-cache` flag
- Changed interface of `L1Handler.execute` and `L1Handler` (dropped `fee` parameter, added result handling with `RevertedTransaction`)
- Contract now has associated state, more about it [here](https://foundry-rs.github.io/starknet-foundry/testing/testing_contract_internals.html)
- cheatcodes (`prank`, `roll`, `warp`) now work on forked Cairo 0 contracts

#### Changed

- Spying events interface is updated to enable the use of events defined inside contracts in assertions
- Test are executed in parallel
- Fixed inconsistent pointers bug https://github.com/foundry-rs/starknet-foundry/issues/659
- Fixed an issue where `deploy_at` would not trigger the constructors https://github.com/foundry-rs/starknet-foundry/issues/805

### Cast

#### Changed

- dropped official support for cairo 1 compiled contracts. While they still should be working without any problems, 
from now on the only officially supported cairo compiler version is 2

## [0.7.1] - 2023-09-27

### Forge

#### Added

- `var` library function for reading environmental variables

#### Fixed
- Using any concrete `block_id` when using forking mode, would lead to crashes 

## [0.7.0] - 2023-09-27

### Forge

#### Added

- Support for scarb workspaces
- Initial version of fuzz testing with randomly generated values
- `#[fork(...)]` attribute allowing testing against a network fork

#### Changed

- Tests are collected only from a package tree (`src/lib.cairo` as an entrypoint) and `tests` folder:
  - If there is a `lib.cairo` file in `tests` folder, then it is treated as an entrypoint to the `tests` package from which tests are collected
  - Otherwise, all test files matching `tests/*.cairo` regex are treated as modules and added to a single virtual `lib.cairo`, which is treated as described above

### Cast

#### Added

- `account add` command for importing accounts to the accounts file
- `account create` command for creating openzeppelin accounts with starkli-style keystore
- `account deploy` command for deploying openzeppelin accounts with starkli-style keystore

### Changed

- `--add-profile` no longer accepts `-a` for short
- allow the `id` property in multicalls to be referenced in the inputs of `deploy` and `invoke` calls

## [0.6.0] - 2023-09-13

### Forge

#### Added

- `deploy_at` cheatcode
- printing failures summary at the end of an execution
- filtering tests now uses an absolute module tree path — it is possible to filter tests by module names, etc.

#### Fixed

- non-zero exit code is returned when any tests fail
- mock_call works with dispatchers if contract does not exists

### Cast

#### Added

- support for starkli-style accounts, allowing the use of existing accounts

#### Changed

- fixed misleading error message when there was no scarb in PATH and `--path-to-scarb-toml` was passed
- modified `multicall new` command output, to be in line with other commands outputs

## [0.5.0] - 2023-08-30

### Forge

#### Added

- support for `keccak_syscall` syscall. It can be used directly in cairo tests
- `l1_handler_execute` cheatcode
- support for `roll`ing/`warp`ing/`prank`ing the constructor logic (precalculate address, prank, assert pranked state in constructor)
- `spy_events` cheatcode
- Functions `read_json` and `FileParser<T>::parse_json` to load data from json files and deserialize it

#### Changed

- rename `TxtParser` trait to `FileParser`
- rename `parse_txt` trait to `read_txt`
- support for printing in contracts
- `spoof` cheatcode
- snforge command-line flag `--init`

### Cast

#### Added

- Support for custom networks - accounts created on custom networks are saved in `accounts-file` under network's
  chain_id
- `accounts-file` field in Scarb.toml profile
- Include the class hash of an account contract in the `accounts-file`

#### Removed

- `--network` option together with the `network` field in Scarb.toml profile — previously used as a validation factor;
  now networks are identified by their chain_id

## [0.4.0] - 2023-08-17

### Forge

#### Added

- `#[should_panic]` attribute support
- Documentation to public methods
- Information sections to documentation about importing `snforge_std`
- Print support for basic numeric data types
- Functions `parse_txt` and `TxtParser<T>::deserialize_txt` to load data from plain text files and serialize it
- `get_class_hash` cheatcode
- `mock_call` cheatcode
- `precalculate_address` cheatcode

#### Changed

- Exported `snforge_std` as a Scarb package, now you have to import it explicitly with e.g. `use snforge_std::declare`
  and add it as a dependency to your Scarb.toml

```toml
[dependencies]
# ...
snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry", tag = "v0.4.0" }
```

- Moved `ForgeConfigFromScarb` to `scarb.rs` and renamed to `ForgeConfig`
- Made private:
    - `print_collected_tests_count`
    - `print_running_tests`
    - `print_test_result`
    - `print_test_summary`
    - `TestCaseSummary::from_run_result`
    - `TestCaseSummary::skipped`
    - `extract_result_data`
    - `StarknetArtifacts`
    - `StarknetContractArtifactPaths`
    - `StarknetContract`
- Split `dependencies_for_package` into separate methods:
    - `paths_for_package`
    - `corelib_for_package`
    - `target_name_for_package`
    - `compilation_unit_for_package`

- Fails test when user tries to use syscalls not supported by forge test runner
- Updated cairo-lang to 2.1.0, starknet-api to 0.4.1 and blockifier to 0.2.0-rc0

### Cast

#### Added

- Added `--class-hash` flag to account create/deploy, allowing for custom openzeppelin account contract class hash

## [0.3.0] - 2023-08-02

### Forge

#### Added

- `warp` cheatcode
- `roll` cheatcode
- `prank` cheatcode
- Most unsafe libfuncs can now be used in contracts

#### Changed

- `declare` return type to `starknet::ClassHash`, doesn't return a `Result`
- `PreparedContract` `class_hash` changed to `starknet::ClassHash`
- `deploy` return type to `starknet::ContractAddress`

#### Fixed

- Using the same cairo file names as corelib files no longer fails test execution

### Cast

#### Added

- multicall as a single transaction
- account creation and deployment
- `--wait` flag to wait for transaction to be accepted/rejected

#### Changed

- sierra and casm artifacts are now required in Scarb.toml for contract declaration
- improved error messages

## [0.1.1] - 2023-07-26

### Forge & Cast

#### Fixed

- `class_hash`es calculation
- Test collection

## [0.1.0] - 2023-07-19

### Forge & Cast

#### Added

- Initial release
