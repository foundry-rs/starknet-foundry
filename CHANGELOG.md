# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Forge

#### Added

- `store` and `load` cheatcodes

#### Fixed

- Error message for tests that should panic but pass

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

- `elect` cheatcode for mocking the sequencer address. Read more [here](./docs/src/appendix/cheatcodes/start_elect.md).
- `--rerun-failed` option to run tests that failed during the last run.

#### Changed
- `start_warp` and `stop_warp` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/start_warp.md). 
- `start_prank` and `stop_prank` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/start_prank.md).
- `start_roll` and `stop_roll` now take `CheatTarget` as the first argument instead of `ContractAddress`. Read more [here](./docs/src/appendix/cheatcodes/start_roll.md).

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
