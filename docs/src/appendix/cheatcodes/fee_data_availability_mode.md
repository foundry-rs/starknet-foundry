# `fee_data_availability_mode`

Cheatcodes modifying `fee_data_availability_mode`:

## `cheat_fee_data_availability_mode`
> `fn cheat_fee_data_availability_mode(contract_address: ContractAddress, fee_data_availability_mode: u32, span: CheatSpan)`

Changes the transaction fee data availability mode for the given target and span.

## `start_cheat_fee_data_availability_mode_global`
> `fn start_cheat_fee_data_availability_mode_global(fee_data_availability_mode: u32)`

Changes the transaction fee data availability mode for all targets.

## `start_cheat_fee_data_availability_mode`
> `fn start_cheat_fee_data_availability_mode(contract_address: ContractAddress, fee_data_availability_mode: u32)`

Changes the transaction fee data availability mode for the given target.

## `stop_cheat_fee_data_availability_mode`
> `fn stop_cheat_fee_data_availability_mode(contract_address: ContractAddress)`

Cancels the `cheat_fee_data_availability_mode` / `start_cheat_fee_data_availability_mode` for the given target.

## `stop_cheat_fee_data_availability_mode_global`
> `fn stop_cheat_fee_data_availability_mode_global()`

Cancels the `start_cheat_fee_data_availability_mode_global`.
