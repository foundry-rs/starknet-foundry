# `signature`

Cheatcodes modifying `signature`:

## `cheat_signature`
> `fn cheat_signature(contract_address: ContractAddress, signature: Span<felt252>, span: CheatSpan)`

Changes the transaction signature for the given target and span.

## `start_cheat_signature_global`
> `fn start_cheat_signature_global(signature: Span<felt252>)`

Changes the transaction signature for all targets.

## `start_cheat_signature`
> `fn start_cheat_signature(contract_address: ContractAddress, signature: Span<felt252>)`

Changes the transaction signature for the given target.

## `stop_cheat_signature`
> `fn stop_cheat_signature(contract_address: ContractAddress)`

Cancels the `cheat_signature` / `start_cheat_signature` for the given target.

## `stop_cheat_signature_global`
> `fn stop_cheat_signature_global()`

Cancels the `start_cheat_signature_global`.
