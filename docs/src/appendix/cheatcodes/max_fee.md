# `max_fee`

Cheatcodes modifying `max_fee`:

## `cheat_max_fee`
> `fn cheat_max_fee(target: ContractAddress, max_fee: u128, span: CheatSpan)`

Changes the transaction max fee for the given target and span.

## `start_cheat_max_fee_global`
> `fn start_cheat_max_fee_global(max_fee: u128)`

Changes the transaction max fee for all targets.

## `start_cheat_max_fee`
> `fn start_cheat_max_fee(target: ContractAddress, max_fee: u128)`

Changes the transaction max fee for the given target.

## `stop_cheat_max_fee`
> `fn stop_cheat_max_fee(target: ContractAddress)`

Cancels the `cheat_max_fee` / `start_cheat_max_fee` for the given target.

## `stop_cheat_max_fee_global`
> `fn stop_cheat_max_fee_global()`

Cancels the `start_cheat_max_fee_global`.
