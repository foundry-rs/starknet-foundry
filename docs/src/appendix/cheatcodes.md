# Cheatcodes Reference

- [`CheatTarget`](cheatcodes/cheat_target.md) - enum for selecting contracts to target with cheatcodes
- [`start_prank`](cheatcodes/start_prank.md) - changes the caller address for contracts
- [`stop_prank`](cheatcodes/stop_prank.md) - cancels the `start_prank` for contracts
- [`start_roll`](cheatcodes/start_roll.md) - changes the block number for contracts
- [`stop_roll`](cheatcodes/stop_roll.md) - cancels the `start_roll` for contracts
- [`start_warp`](cheatcodes/start_warp.md) - changes the block timestamp for contracts
- [`stop_warp`](cheatcodes/stop_warp.md) - cancels the `start_warp` for contracts
- [`start_elect`](cheatcodes/start_elect.md) - changes the sequencer address for contracts
- [`stop_elect`](cheatcodes/stop_elect.md) - cancels the `start_elect` for contracts
- [`start_spoof`](cheatcodes/start_spoof.md) - changes the transaction context for contracts
- [`stop_spoof`](cheatcodes/stop_spoof.md) - cancels the `start_spoof` for contracts
- [`get_class_hash`](cheatcodes/get_class_hash.md) - retrieves a class hash of a contract
- [`start_mock_call`](cheatcodes/start_mock_call.md) - mocks contract call to a `function_name` of a contract
- [`stop_mock_call`](cheatcodes/stop_mock_call.md) - cancels the `start_mock_call` for the function `function_name` of a contract
- [`l1_handler_execute`](cheatcodes/l1_handler_execute.md) - executes a `#[l1_handler]` function to mock a message arriving from Ethereum
- [`spy_events`](cheatcodes/spy_events.md) - creates `EventSpy` instance which spies on events emitted by contracts
- [`store`](cheatcodes/store.md) - stores values in targeted contact's storage
- [`load`](cheatcodes/load.md) - loads values directly from targeted contact's storage

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>
> ```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```
