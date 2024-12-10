# `block_hash`

Cheatcodes modifying `block_hash`:

## `cheat_block_hash`
> `fn cheat_block_hash(target: ContractAddress, block_hash: felt252, span: CheatSpan)`

Changes the block hash for the given target and span.

## `start_cheat_block_hash_global`
> `fn start_cheat_block_hash_global(block_hash: felt252)`

Changes the block hash for all targets.

## `start_cheat_block_hash`
> `fn start_cheat_block_hash(target: ContractAddress, block_hash: felt252)`

Changes the block hash for the given target.

## `stop_cheat_block_hash`
> `fn stop_cheat_block_hash(target: ContractAddress)`

Cancels the `cheat_block_hash` / `start_cheat_block_hash` for the given target.

## `stop_cheat_block_hash_global`
> `fn stop_cheat_block_hash_global()`

Cancels the `start_cheat_block_hash_global`.

