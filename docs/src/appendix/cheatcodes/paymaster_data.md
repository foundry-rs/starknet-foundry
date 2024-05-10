# `paymaster_data`

Cheatcodes modifying `paymaster_data`:

## `cheat_paymaster_data`
> `fn cheat_paymaster_data(target: ContractAddress, paymaster_data: Span<felt252>, span: CheatSpan)`

Changes the transaction paymaster data for the given target and span.

## `cheat_paymaster_data_global`
> `fn cheat_paymaster_data_global(paymaster_data: Span<felt252>)`

Changes the transaction paymaster data for all targets.

## `start_cheat_paymaster_data`
> `fn start_cheat_paymaster_data(target: ContractAddress, paymaster_data: Span<felt252>)`

Changes the transaction paymaster data for the given target.

# `stop_cheat_paymaster_data`
> `fn stop_cheat_paymaster_data(target: ContractAddress)`

Cancels the `cheat_paymaster_data` / `start_cheat_paymaster_data` for the given target.

# `stop_cheat_paymaster_data_global`
> `fn stop_cheat_paymaster_data_global(target: ContractAddress)`

Cancels the `cheat_paymaster_data_global`.
