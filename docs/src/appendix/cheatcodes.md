# Cheatcodes Reference

- [`start_prank`](cheatcodes/start_prank.md) - changes the caller address for a contract
- [`stop_prank`](cheatcodes/stop_prank.md) - cancels the `start_prank` for the contract
- [`start_roll`](cheatcodes/start_roll.md) - changes the block number for a contract
- [`stop_roll`](cheatcodes/stop_roll.md) - cancels the `start_roll` for the contract
- [`start_warp`](cheatcodes/start_warp.md) - changes the block timestamp for a contract
- [`stop_warp`](cheatcodes/stop_warp.md) - cancels the `start_warp` for the contract
- [`get_class_hash`](cheatcodes/get_class_hash.md) - retrieves a class hash of a contract
- [`start_mock_call`](cheatcodes/start_mock_call.md) - mocks contract call to a `function_name` of a contract
- [`stop_mock_call`](cheatcodes/stop_mock_call.md) - cancels the `start_mock_call` for the function `function_name` of a contract
- [`l1_handler_execute`](cheatcodes/l1_handler_execute.md) - executes a `#[l1_handler]` function to mock a message arriving from Ethereum
- [`spy_events`](cheatcodes/spy_events.md) - creates `EventSpy` instance which spies on events emitted by contracts

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>
> ```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.7.1" }
> ```
