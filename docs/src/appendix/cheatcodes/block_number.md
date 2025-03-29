# `block_number`

Cheatcodes modifying `block_number`:

## `cheat_block_number`
> `fn cheat_block_number(contract_address: ContractAddress, block_number: u64, span: CheatSpan)`

Changes the block number for the given contract address and span.

## `start_cheat_block_number_global`
> `fn start_cheat_block_number_global(block_number: u64)`

Changes the block number for all contract addresses.

## `start_cheat_block_number`
> `fn start_cheat_block_number(contract_address: ContractAddress, block_number: u64)`

Changes the block number for the given contract address.

## `stop_cheat_block_number`
> `fn stop_cheat_block_number(contract_address: ContractAddress)`

Cancels the `cheat_block_number` / `start_cheat_block_number` for the given contract address.

## `stop_cheat_block_number_global`
> `fn stop_cheat_block_number_global()`

Cancels the `start_cheat_block_number_global`.

