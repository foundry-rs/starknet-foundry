# Cheatcodes Reference

- [`CheatTarget`](cheatcodes/cheat_target.md) - enum for selecting contracts to target with cheatcodes
- [`CheatSpan`](cheatcodes/cheat_span.md) - enum for specifying the number of target calls for a cheat
- [`prank`](cheatcodes/caller_address#prank) - changes the caller address for contracts, for a number of calls
- [`start_prank`](cheatcodes/caller_address#start_prank) - changes the caller address for contracts
- [`stop_prank`](cheatcodes/caller_address#stop_prank) - cancels the `prank` / `start_prank` for contracts
- [`roll`](cheatcodes/block_number#roll) - changes the block number for contracts, for a number of calls
- [`start_roll`](cheatcodes/block_number#start_roll) - changes the block number for contracts
- [`stop_roll`](cheatcodes/block_number#stop_roll) - cancels the `roll` / `start_roll` for contracts
- [`warp`](cheatcodes/block_timestamp#warp) - changes the block timestamp for contracts, for a number of calls
- [`start_warp`](cheatcodes/block_timestamp#start_warp) - changes the block timestamp for contracts
- [`stop_warp`](cheatcodes/block_timestamp#stop_warp) - cancels the `warp` / `start_warp` for contracts
- [`elect`](cheatcodes/sequencer_address/elect.md) - changes the sequencer address for contracts, for a number of calls
- [`start_elect`](cheatcodes/sequencer_address/start_elect.md) - changes the sequencer address for contracts
- [`stop_elect`](cheatcodes/sequencer_address/stop_elect.md) - cancels the `elect` / `start_elect` for contracts
- [`spoof`](cheatcodes/tx_info#spoof) - changes the transaction context for contracts, for a number of calls
- [`start_spoof`](cheatcodes/tx_info#start_spoof) - changes the transaction context for contracts
- [`stop_spoof`](cheatcodes/tx_info#stop_spoof) - cancels the `spoof` / `start_spoof` for contracts
- [`mock_call`](cheatcodes/mock_call.md#mock_call) - mocks a number of contract calls to an entry point
- [`start_mock_call`](cheatcodes/mock_call.md#start_mock_call) - mocks contract call to an entry point
- [`stop_mock_call`](cheatcodes/mock_call.md#stop_mock_call) - cancels the `mock_call` / `start_mock_call` for an entry point
- [`get_class_hash`](cheatcodes/get_class_hash.md) - retrieves a class hash of a contract
- [`replace_bytecode`](cheatcodes/replace_bytecode.md) - replace the class hash of a contract
- [`l1_handler_execute`](cheatcodes/l1_handler_execute.md) - executes a `#[l1_handler]` function to mock a message arriving from Ethereum
- [`spy_events`](cheatcodes/spy_events.md) - creates `EventSpy` instance which spies on events emitted by contracts
- [`store`](cheatcodes/store.md) - stores values in targeted contact's storage
- [`load`](cheatcodes/load.md) - loads values directly from targeted contact's storage

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a development dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using appropriate release tag.
>
> ```toml
> [dev-dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```
