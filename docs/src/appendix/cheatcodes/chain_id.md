# `chain_id`

Cheatcodes modifying `chain_id`:

## `cheat_chain_id`
> `fn cheat_chain_id(contract_address: ContractAddress, chain_id: felt252, span: CheatSpan)`

Changes the transaction chain_id for the given contract address and span.

## `start_cheat_chain_id_global`
> `fn start_cheat_chain_id_global(chain_id: felt252)`

Changes the transaction chain_id for all contract addresses.

## `start_cheat_chain_id`
> `fn start_cheat_chain_id(contract_address: ContractAddress, chain_id: felt252)`

Changes the transaction chain_id for the given contract address.

## `stop_cheat_chain_id`
> `fn stop_cheat_chain_id(contract_address: ContractAddress)`

Cancels the `cheat_chain_id` / `start_cheat_chain_id` for the given contract addresses.

## `stop_cheat_chain_id_global`
> `fn stop_cheat_chain_id_global()`

Cancels the `start_cheat_chain_id_global`.
