# `caller_address`

Cheatcodes modifying `caller_address`:

## `cheat_caller_address`
> `fn cheat_caller_address(contract_address: ContractAddress, caller_address: ContractAddress, span: CheatSpan)`

Changes the caller address for the given target and span.

## `start_cheat_caller_address_global`
> `fn start_cheat_caller_address_global(caller_address: ContractAddress)`

Changes the caller address for all targets.

## `start_cheat_caller_address`
> `fn start_cheat_caller_address(contract_address: ContractAddress, caller_address: ContractAddress)`

Changes the caller address for the given target.

## `stop_cheat_caller_address`
> `fn stop_cheat_caller_address(contract_address: ContractAddress)`

Cancels the `cheat_caller_address` / `start_cheat_caller_address` for the given target.

## `stop_cheat_caller_address_global`
> `fn stop_cheat_caller_address_global()`

Cancels the `start_cheat_caller_address_global`.
