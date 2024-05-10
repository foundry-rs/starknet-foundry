# `version`

Cheatcodes modifying `version`:

## `cheat_version`
> `fn cheat_version(target: ContractAddress, version: felt252, span: CheatSpan)`

Changes the transaction version for the given target and span.

## `cheat_version_global`
> `fn cheat_version_global(version: felt252)`

Changes the transaction version for all targets.

## `start_cheat_version`
> `fn start_cheat_version(target: ContractAddress, version: felt252)`

Changes the transaction version for the given target.

# `stop_cheat_version`
> `fn stop_cheat_version(target: ContractAddress)`

Cancels the `cheat_version` / `start_cheat_version` for the given target.

# `stop_cheat_version_global`
> `fn stop_cheat_version_global(target: ContractAddress)`

Cancels the `cheat_version_global`.
