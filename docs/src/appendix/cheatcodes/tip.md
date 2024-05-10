# `tip`

Cheatcodes modifying `tip`:

## `cheat_tip`
> `fn cheat_tip(target: ContractAddress, tip: u128, span: CheatSpan)`

Changes the transaction tip for the given target and span.

## `cheat_tip_global`
> `fn cheat_tip_global(tip: u128)`

Changes the transaction tip for all targets.

## `start_cheat_tip`
> `fn start_cheat_tip(target: ContractAddress, tip: u128)`

Changes the transaction tip for the given target.

# `stop_cheat_tip`
> `fn stop_cheat_tip(target: ContractAddress)`

Cancels the `cheat_tip` / `start_cheat_tip` for the given target.

# `stop_cheat_tip_global`
> `fn stop_cheat_tip_global(target: ContractAddress)`

Cancels the `cheat_tip_global`.
