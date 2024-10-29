# Summary

[Introduction](./README.md)

# Getting Started

* [Installation](getting-started/installation.md)
* [First Steps with Starknet Foundry](getting-started/first-steps.md)
* [Scarb](getting-started/scarb.md)
* [Project Configuration](projects/configuration.md)

---

# `snforge` Overview

* [Running Tests](testing/running-tests.md)
* [Writing Tests](testing/testing.md)
* [Test Attributes](testing/test-attributes.md)
* [Testing Smart Contracts](testing/contracts.md)
* [Testing Contracts' Internals](testing/testing-contract-internals.md)
* [Using Cheatcodes](testing/using-cheatcodes.md)
* [Testing Events](testing/testing-events.md)
* [Testing Messages to L1](testing/testing-messages-to-l1.md)
* [Testing Workspaces](testing/testing-workspaces.md)
* [Test Collection](testing/test-collection.md)
* [Contract Collection](testing/contracts-collection.md)
* [Gas and VM Resources Estimation](testing/gas-and-resource-estimation.md)
* [Coverage](testing/coverage.md)

# `snforge` Advanced Features

* [Fork Testing](snforge-advanced-features/fork-testing.md)
* [Fuzz Testing](snforge-advanced-features/fuzz-testing.md)
* [Conditional Compilation](snforge-advanced-features/conditional-compilation.md)
* [Direct Storage Access](snforge-advanced-features/storage-cheatcodes.md)
* [Profiling](snforge-advanced-features/profiling.md)

---

# `sncast` Overview

* [Outline](starknet/index.md)
* [Creating And Deploying Accounts](starknet/account.md)
* [Importing Accounts](starknet/account-import.md)
* [Declaring New Contracts](starknet/declare.md)
* [Deploying New Contracts](starknet/deploy.md)
* [Invoking Contracts](starknet/invoke.md)
* [Calling Contracts](starknet/call.md)
* [Performing Multicall](starknet/multicall.md)
* [Cairo Deployment Scripts](starknet/script.md)
* [Inspecting Transactions](starknet/tx-status.md)
* [Fees and Versions](starknet/fees-and-versions.md)
* [Verifying Contracts](starknet/verify.md)
* [Calldata Transformation](starknet/calldata-transformation.md)

---

# Foundry Development

* [Environment Setup](development/environment-setup.md)

---

# Appendix

* [`snforge` Commands](appendix/snforge.md)
    * [test](appendix/snforge/test.md)
    * [init](appendix/snforge/init.md)
    * [clean-cache](appendix/snforge/clean-cache.md)
* [Cheatcodes Reference](appendix/cheatcodes.md)
    * [Cheating Globally](appendix/cheatcodes/global.md)
    * [CheatSpan](appendix/cheatcodes/cheat_span.md)
    * [caller_address](appendix/cheatcodes/caller_address.md)
    * [block_number](appendix/cheatcodes/block_number.md)
    * [block_timestamp](appendix/cheatcodes/block_timestamp.md)
    * [sequencer_address](appendix/cheatcodes/sequencer_address.md)
    * [version](appendix/cheatcodes/transaction_version.md)
    * [account_contract_address](appendix/cheatcodes/account_contract_address.md)
    * [max_fee](appendix/cheatcodes/max_fee.md)
    * [signature](appendix/cheatcodes/signature.md)
    * [transaction_hash](appendix/cheatcodes/transaction_hash.md)
    * [chain_id](appendix/cheatcodes/chain_id.md)
    * [nonce](appendix/cheatcodes/nonce.md)
    * [resource_bounds](appendix/cheatcodes/resource_bounds.md)
    * [tip](appendix/cheatcodes/tip.md)
    * [paymaster_data](appendix/cheatcodes/paymaster_data.md)
    * [nonce_data_availability_mode](appendix/cheatcodes/nonce_data_availability_mode.md)
    * [fee_data_availability_mode](appendix/cheatcodes/fee_data_availability_mode.md)
    * [account_deployment_data](appendix/cheatcodes/account_deployment_data.md)
    * [mock_call](appendix/cheatcodes/mock_call.md)
    * [get_class_hash](appendix/cheatcodes/get_class_hash.md)
    * [replace_bytecode](appendix/cheatcodes/replace_bytecode.md)
    * [l1_handler](appendix/cheatcodes/l1_handler.md)
    * [spy_events](appendix/cheatcodes/spy_events.md)
    * [spy_messages_to_l1](appendix/cheatcodes/spy_messages_to_l1.md)
    * [store](appendix/cheatcodes/store.md)
    * [load](appendix/cheatcodes/load.md)
* [`snforge` Library Reference](appendix/snforge-library.md)
    * [byte_array](appendix/snforge-library/byte_array.md)
    * [declare](appendix/snforge-library/declare.md)
    * [contract_class](appendix/snforge-library/contract_class.md)
    * [get_call_trace](appendix/snforge-library/get_call_trace.md)
    * [fs](appendix/snforge-library/fs.md)
    * [env](appendix/snforge-library/env.md)
    * [signature](appendix/snforge-library/signature.md)
* [`sncast` Commands](appendix/sncast.md)
    * [common flags](appendix/sncast/common.md)
    * [account](appendix/sncast/account/account.md)
        * [import](appendix/sncast/account/import.md)
        * [create](appendix/sncast/account/create.md)
        * [deploy](appendix/sncast/account/deploy.md)
        * [delete](appendix/sncast/account/delete.md)
        * [list](appendix/sncast/account/list.md)
    * [declare](appendix/sncast/declare.md)
    * [deploy](appendix/sncast/deploy.md)
    * [invoke](appendix/sncast/invoke.md)
    * [call](appendix/sncast/call.md)
    * [multicall](appendix/sncast/multicall/multicall.md)
        * [new](appendix/sncast/multicall/new.md)
        * [run](appendix/sncast/multicall/run.md)
    * [show-config](appendix/sncast/show_config.md)
    * [script](appendix/sncast/script/script.md)
        * [init](appendix/sncast/script/init.md)
        * [run](appendix/sncast/script/run.md)
    * [tx-status](appendix/sncast/tx-status.md)
    * [verify](appendix/sncast/verify.md)
* [`sncast` Library Reference](appendix/sncast-library.md)
    * [declare](appendix/sncast-library/declare.md)
    * [deploy](appendix/sncast-library/deploy.md)
    * [invoke](appendix/sncast-library/invoke.md)
    * [call](appendix/sncast-library/call.md)
    * [get_nonce](appendix/sncast-library/get_nonce.md)
    * [tx_status](appendix/sncast-library/tx_status.md)
    * [errors](appendix/sncast-library/errors.md)
* [ `snfoundry.toml` Reference](appendix/snfoundry-toml.md)
* [ `Scarb.toml` Reference](appendix/scarb-toml.md)
