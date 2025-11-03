# Roadmap

Tentative roadmap for Starknet Foundry. This document reflects the state of the roadmap at the time of writing.
We strive for our roadmap to reflect user needs and feedback, expect changes to this document.

**All feedback and request welcome.**

## Table of Contents

<!-- TOC -->
* [Roadmap](#roadmap)
  * [Table of Contents](#table-of-contents)
  * [Reference](#reference)
  * [Forge](#forge)
    * [🏗 Test Partitioning](#-test-partitioning)
    * [🏗 Asserting Steps in Execution](#-asserting-steps-in-execution)
    * [Reverting Storage Changes in Execution](#reverting-storage-changes-in-execution)
    * [🏗 Sierra -> Casm Compilation](#-sierra---casm-compilation)
    * [Performance Investigation](#performance-investigation)
    * [Advanced Forking / Forking State Asserting](#advanced-forking--forking-state-asserting)
    * [Derive Macro for `Fuzzable` Trait](#derive-macro-for-fuzzable-trait)
    * [Typesafe Contract "declare"](#typesafe-contract-declare)
    * [Research Variant and Differential Testing, Better Fuzzing Algorithms](#research-variant-and-differential-testing-better-fuzzing-algorithms)
  * [Cast](#cast)
    * [New Cast Scripts](#new-cast-scripts)
    * [Transaction Dry Run](#transaction-dry-run)
    * [CLI Revamp and Configuration Refactor](#cli-revamp-and-configuration-refactor)
    * [Better Accounts Support](#better-accounts-support)
    * [New Multicall Interface](#new-multicall-interface)
    * [Contract Aliases in `snfoundry.toml`](#contract-aliases-in-snfoundrytoml)
<!-- TOC -->

## Reference

* Item "size" is in the scale 1 to 5 and reflects its relative complexity compared to other items in the roadmap.
* Items marked with 🏗️ are in progress.
* Items marked with ✅ are done.

## Forge

### 🏗 Test Partitioning

_Size: 3_

https://github.com/foundry-rs/starknet-foundry/issues/3548

Partitioning test suite into smaller test suites, to be run on separate machines in CI. Similar to `cargo nextest`.

### 🏗 Asserting Steps in Execution

_Size: 2_

https://github.com/foundry-rs/starknet-foundry/issues/2671

Feature for asserting the number of steps used in test execution.

### Reverting Storage Changes in Execution

_Size: 3_

https://github.com/foundry-rs/starknet-foundry/issues/3837

Change the test execution model to revert storage changes from top-level calls in case of recoverable failure.

### 🏗 Sierra -> Casm Compilation

_Size: 3_

https://github.com/foundry-rs/starknet-foundry/issues/3832

Sierra -> Casm performance investigation and optimization (if viable).

### Performance Investigation

_Size: 3_

https://github.com/foundry-rs/starknet-foundry/issues/3899

Investigate bottlenecks in standard test execution (workspace & package processing, config run, collecting configs, test
execution) using tracing harnesses. Performance report on eventual findings and measurements for future optimizations.

### Advanced Forking / Forking State Asserting

_Size: 5_

New test mechanism for detecting regressions in new contract versions (for upgrades on chain). Forking and asserting
state changes after executing a test scenario.

### Derive Macro for `Fuzzable` Trait

_Size: 2_

https://github.com/foundry-rs/starknet-foundry/issues/2968

Ability to automatically derive `Fuzzable` trait for structs if they contain only `Fuzzable` fields.

### Typesafe Contract "declare"

_Size: 4_

https://github.com/foundry-rs/starknet-foundry/issues/1531

Detect and fail on invalid contract names at compilation time.

### Research Variant and Differential Testing, Better Fuzzing Algorithms

_Size: 5_

https://github.com/foundry-rs/starknet-foundry/issues/2464

Inspired by features from Ethereum's Foundry, research the viability of adding variant and differential testing and
integrating better
fuzzing algorithms.

## Cast

### New Cast Scripts

_Size: 5_

https://github.com/foundry-rs/starknet-foundry/issues/3523

New Cast Scripts with focus on the ease of use, using Scarb plugins, integrated into snforge/scarb tests structure.

### Transaction Dry Run

_Size: 1_

https://github.com/foundry-rs/starknet-foundry/issues/2136

Running `sncast` transaction without executing them through the fee estimation endpoint.

### CLI Revamp and Configuration Refactor

_Size: 4_

Removing non-common arguments that are used as common (e.g. `-account`). Internal changes to how `sncast` loads and
combines configuration.

### Better Accounts Support

_Size: 4_

Native ledger and keystore support with encryption, account storage rework.

### New Multicall Interface

_Size: 3_

https://github.com/foundry-rs/starknet-foundry/issues/3810

Native multicall support for invoking transactions in `sncast invoke` or a better dedicated command. Removal of
multicall files.

### Contract Aliases in `snfoundry.toml`

_Size: 1_

https://github.com/foundry-rs/starknet-foundry/issues/2240

Aliases for contracts in `snfoundry.toml` that can be used in commands instead of contract addresses.
