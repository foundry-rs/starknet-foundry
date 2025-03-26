# `nonce`

Cheatcodes modifying `nonce`:

## `cheat_nonce`
> `fn cheat_nonce(contract_address: ContractAddress, nonce: felt252, span: CheatSpan)`

Changes the transaction nonce for the given contract address and span.

## `start_cheat_nonce_global`
> `fn start_cheat_nonce_global(nonce: felt252)`

Changes the transaction nonce for all contract addresses.

## `start_cheat_nonce`
> `fn start_cheat_nonce(contract_address: ContractAddress, nonce: felt252)`

Changes the transaction nonce for the given contract address.

## `stop_cheat_nonce`
> `fn stop_cheat_nonce(contract_address: ContractAddress)`

Cancels the `cheat_nonce` / `start_cheat_nonce` for the given contract address.

## `stop_cheat_nonce_global`
> `fn stop_cheat_nonce_global()`

Cancels the `start_cheat_nonce_global`.
