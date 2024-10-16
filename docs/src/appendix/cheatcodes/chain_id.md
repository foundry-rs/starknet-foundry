# `chain_id`

Cheatcodes modifying `chain_id`:

## `cheat_chain_id`
> `fn cheat_chain_id(target: ContractAddress, chain_id: felt252, span: CheatSpan)`

Changes the transaction chain_id for the given target and span.

## `start_cheat_chain_id_global`
> `fn start_cheat_chain_id_global(chain_id: felt252)`

Changes the transaction chain_id for all targets.

## `start_cheat_chain_id`
> `fn start_cheat_chain_id(target: ContractAddress, chain_id: felt252)`

Changes the transaction chain_id for the given target.

## `stop_cheat_chain_id`
> `fn stop_cheat_chain_id(target: ContractAddress)`

Cancels the `cheat_chain_id` / `start_cheat_chain_id` for the given target.

## `stop_cheat_chain_id_global`
> `fn stop_cheat_chain_id_global()`

Cancels the `start_cheat_chain_id_global`.
