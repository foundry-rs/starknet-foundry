# `tip`

Cheatcodes modifying `tip`:

## `cheat_tip`
> `fn cheat_tip(contract_address: ContractAddress, tip: u128, span: CheatSpan)`

Changes the transaction tip for the given contract address and span.

## `start_cheat_tip_global`
> `fn start_cheat_tip_global(tip: u128)`

Changes the transaction tip for all contract addresses.

## `start_cheat_tip`
> `fn start_cheat_tip(contract_address: ContractAddress, tip: u128)`

Changes the transaction tip for the given contract address.

## `stop_cheat_tip`
> `fn stop_cheat_tip(contract_address: ContractAddress)`

Cancels the `cheat_tip` / `start_cheat_tip` for the given contract address.

## `stop_cheat_tip_global`
> `fn stop_cheat_tip_global()`

Cancels the `start_cheat_tip_global`.
