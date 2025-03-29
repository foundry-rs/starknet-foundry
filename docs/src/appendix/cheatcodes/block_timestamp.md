# `block_timestamp`

Cheatcodes modifying `block_timestamp`:

## `cheat_block_timestamp`
> `fn cheat_block_timestamp(contract_address: ContractAddress, block_timestamp: u64, span: CheatSpan)`

Changes the block timestamp for the given contract address and span.

## `start_cheat_block_timestamp_global`
> `fn start_cheat_block_timestamp_global(block_timestamp: u64)`

Changes the block timestamp for all contract addresses.

## `start_cheat_block_timestamp`
> `fn start_cheat_block_timestamp(contract_address: ContractAddress, block_timestamp: u64)`

Changes the block timestamp for the given contract address.

## `stop_cheat_block_timestamp`
> `fn stop_cheat_block_timestamp(contract_address: ContractAddress)`

Cancels the `cheat_block_timestamp` / `start_cheat_block_timestamp` for the given contract address.

## `stop_cheat_block_timestamp_global`
> `fn stop_cheat_block_timestamp_global()`

Cancels the `start_cheat_block_timestamp_global`.
