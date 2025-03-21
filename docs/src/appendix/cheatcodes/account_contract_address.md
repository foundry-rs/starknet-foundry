# `account_contract_address`

Cheatcodes modifying `account_contract_address`:

## `cheat_account_contract_address`
> `fn cheat_account_contract_address(contract_address: ContractAddress, account_contract_address: ContractAddress, span: CheatSpan)`

Changes the address of an account which the transaction originates from, for the given target and span.

## `start_cheat_account_contract_address_global`
> `fn start_cheat_account_contract_address_global(account_contract_address: ContractAddress)`

Changes the address of an account which the transaction originates from, for all targets.

## `start_cheat_account_contract_address`
> `fn start_cheat_account_contract_address(contract_address: ContractAddress, account_contract_address: ContractAddress)`

Changes the address of an account which the transaction originates from, for the given target.

## `stop_cheat_account_contract_address`
> `fn stop_cheat_account_contract_address(contract_address: ContractAddress)`

Cancels the `cheat_account_contract_address` / `start_cheat_account_contract_address` for the given target.

## `stop_cheat_account_contract_address_global`
> `fn stop_cheat_account_contract_address_global()`

Cancels the `start_cheat_account_contract_address_global`.
