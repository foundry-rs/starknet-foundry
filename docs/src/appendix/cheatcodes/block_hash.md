# `block_hash`

Cheatcodes modifying `get_block_hash_syscall` output:

## `cheat_block_hash`

> `fn cheat_block_hash(contract_address: ContractAddress, block_number: u64, block_hash: felt252, span: CheatSpan)`

Changes the block hash for the given block number.

## `start_cheat_block_hash_global`

> `fn start_cheat_block_hash_global(block_number: u64, block_hash: felt252)`

Globally modifies the block hash for a specified block number until explicitly stopped.

## `stop_cheat_block_hash_global`

> `fn stop_cheat_block_hash_global(block_number: u64)`

Stops a global block hash modification.

## `start_cheat_block_hash`

> `fn start_cheat_block_hash(contract_address: ContractAddress, block_number: u64, block_hash: felt252)`

Modifies the block hash for a given block number and contract until the cheat is explicitly stopped.

## `stop_cheat_block_hash`

> `fn stop_cheat_block_hash(contract_address: ContractAddress, block_number: u64)`

Stops an active block hash modification for a specific contract.
