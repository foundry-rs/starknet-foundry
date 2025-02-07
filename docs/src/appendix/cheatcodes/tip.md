# `tip`

Cheatcodes modifying `tip`:

## `cheat_tip`
> `fn cheat_tip(target: ContractAddress, tip: u128, span: CheatSpan)`

Changes the transaction tip for the given target and span.

## `start_cheat_tip_global`
> `fn start_cheat_tip_global(tip: u128)`

Changes the transaction tip for all targets.

## `start_cheat_tip`
> `fn start_cheat_tip(target: ContractAddress, tip: u128)`

Changes the transaction tip for the given target.

## `stop_cheat_tip`
> `fn stop_cheat_tip(target: ContractAddress)`

Cancels the `cheat_tip` / `start_cheat_tip` for the given target.

## `stop_cheat_tip_global`
> `fn stop_cheat_tip_global()`

Cancels the `start_cheat_tip_global`.
