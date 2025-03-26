# `account_deployment_data`

Cheatcodes modifying `account_deployment_data`:

## `cheat_account_deployment_data`
> `fn cheat_account_deployment_data(contract_address: ContractAddress, account_deployment_data: Span<felt252>, span: CheatSpan)`

Changes the transaction account deployment data for the given contract address and span.

## `start_cheat_account_deployment_data_global`
> `fn start_cheat_account_deployment_data_global(account_deployment_data: Span<felt252>)`

Changes the transaction account deployment data for all contract addresses.

## `start_cheat_account_deployment_data`
> `fn start_cheat_account_deployment_data(contract_address: ContractAddress, account_deployment_data: Span<felt252>)`

Changes the transaction account deployment data for the given contract address.

## `stop_cheat_account_deployment_data`
> `fn stop_cheat_account_deployment_data(contract_address: ContractAddress)`

Cancels the `cheat_account_deployment_data` / `start_cheat_account_deployment_data` for the given contract address.

## `stop_cheat_account_deployment_data_global`
> `fn stop_cheat_account_deployment_data_global()`

Cancels the `start_cheat_account_deployment_data_global`.
