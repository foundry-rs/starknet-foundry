# `nonce`

Cheatcodes modifying `nonce`:

## `cheat_nonce`
> `fn cheat_nonce(target: ContractAddress, nonce: felt252, span: CheatSpan)`

Changes the transaction nonce for the given target and span.

## `start_cheat_nonce_global`
> `fn start_cheat_nonce_global(nonce: felt252)`

Changes the transaction nonce for all targets.

## `start_cheat_nonce`
> `fn start_cheat_nonce(target: ContractAddress, nonce: felt252)`

Changes the transaction nonce for the given target.

## `stop_cheat_nonce`
> `fn stop_cheat_nonce(target: ContractAddress)`

Cancels the `cheat_nonce` / `start_cheat_nonce` for the given target.

## `stop_cheat_nonce_global`
> `fn stop_cheat_nonce_global()`

Cancels the `start_cheat_nonce_global`.
