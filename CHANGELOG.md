# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Forge

#### Added

- support for `deploy_syscall` directly in the test code (alternative to `deploy`)

#### Changed

- `snforge_std::PrintTrait` will not convert values representing ASCII control characters to strings

#### Added

- `--cores` flag to specify the number of cores to use for parallel execution

### Cast

#### Added

- `show-config` subcommand to display currently used configuration

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

## [0.7.1] - 2023-09-27

### Forge

#### Added

- `var` library function for reading environmental variables

### Fixed
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

### Fixed

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
