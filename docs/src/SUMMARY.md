# Summary

[Introduction](./README.md)

# Getting Started

* [Installation](getting-started/installation.md)
* [First Steps with Starknet Foundry](getting-started/first-steps.md)
* [Scarb](getting-started/scarb.md)
* [Project Configuration](projects/configuration.md)

# Forge overview

* [Running Tests](testing/running-tests.md)
* [Writing Tests](testing/testing.md)
* [Testing Smart Contracts](testing/contracts.md)
* [Testing Contracts' Internals](testing/testing_contract_internals.md)
* [Fork Testing](testing/fork-testing.md)
* [Using Cheatcodes](testing/using-cheatcodes.md)
* [Debugging](testing/debugging.md)
* [Fuzz Testing](testing/fuzz-testing.md)
* [Test Collection](testing/test-collection.md)

# Cast Overview

* [Outline](starknet/index.md)
* [Creating And Deploying Accounts](starknet/account.md)
* [Declaring New Contracts](starknet/declare.md)
* [Deploying New Contracts](starknet/deploy.md)
* [Invoking Contracts](starknet/invoke.md)
* [Calling Contracts](starknet/call.md)
* [Performing Multicall](starknet/multicall.md)

# Foundry Development

* [Environment Setup](development/environment-setup.md)

# Appendix

* [Forge Commands](appendix/forge.md)
    * [test](appendix/forge/test.md)
    * [init](appendix/forge/init.md)
    * [clean-cache](appendix/forge/clean-cache.md)
* [Cheatcodes Reference](appendix/cheatcodes.md)
    * [start_prank](appendix/cheatcodes/start_prank.md)
    * [stop_prank](appendix/cheatcodes/stop_prank.md)
    * [start_roll](appendix/cheatcodes/start_roll.md)
    * [stop_roll](appendix/cheatcodes/stop_roll.md)
    * [start_warp](appendix/cheatcodes/start_warp.md)
    * [stop_warp](appendix/cheatcodes/stop_warp.md)
    * [get_class_hash](appendix/cheatcodes/get_class_hash.md)
    * [l1_handler_execute](appendix/cheatcodes/l1_handler_execute.md)
    * [start_mock_call](appendix/cheatcodes/start_mock_call.md)
    * [stop_mock_call](appendix/cheatcodes/stop_mock_call.md)
    * [spy_events](appendix/cheatcodes/spy_events.md)
    * [start_spoof](appendix/cheatcodes/start_spoof.md)
    * [stop_spoof](appendix/cheatcodes/stop_spoof.md)

* [Forge Library Functions References](appendix/forge-library.md)
    * [declare](appendix/forge-library/declare.md)
    * [precalculate_address](appendix/forge-library/precalculate_address.md)
    * [deploy](appendix/forge-library/deploy.md)
    * [deploy_at](appendix/forge-library/deploy_at.md)
    * [print](appendix/forge-library/print.md)
    * [fs](appendix/forge-library/fs.md)
        * [read_txt](appendix/forge-library/fs/read_txt.md)
        * [parse_txt](appendix/forge-library/fs/parse_txt.md)
        * [read_json](appendix/forge-library/fs/read_json.md)
        * [parse_json](appendix/forge-library/fs/parse_json.md)
    * [env](appendix/forge-library/env.md)
        * [var](appendix/forge-library/env/var.md)
    * [signature](appendix/forge-library/signature.md)
        * [Interface](appendix/forge-library/signature/interface.md)
        * [StarkCurveKeyPair](appendix/forge-library/signature/stark_curve_key_pair.md)
* [Cast Commands](appendix/cast.md)
    * [common flags](appendix/cast/common.md)
    * [account](appendix/cast/account/account.md)
        * [add](appendix/cast/account/add.md)
        * [create](appendix/cast/account/create.md)
        * [deploy](appendix/cast/account/deploy.md)
        * [delete](appendix/cast/account/delete.md)
    * [declare](appendix/cast/declare.md)
    * [deploy](appendix/cast/deploy.md)
    * [invoke](appendix/cast/invoke.md)
    * [call](appendix/cast/call.md)
    * [multicall](appendix/cast/multicall/multicall.md)
        * [new](appendix/cast/multicall/new.md)
        * [run](appendix/cast/multicall/run.md)
    * [show-config](appendix/cast/show_config.md)
