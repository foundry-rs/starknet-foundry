# `nonce_data_availability_mode`

Cheatcodes modifying `nonce_data_availability_mode`:

## `cheat_nonce_data_availability_mode`
> `fn cheat_nonce_data_availability_mode(target: ContractAddress, nonce_data_availability_mode: u32, span: CheatSpan)`

Changes the transaction nonce data availability mode for the given target and span.

## `start_cheat_nonce_data_availability_mode_global`
> `fn start_cheat_nonce_data_availability_mode_global(nonce_data_availability_mode: u32)`

Changes the transaction nonce data availability mode for all targets.

## `start_cheat_nonce_data_availability_mode`
> `fn start_cheat_nonce_data_availability_mode(target: ContractAddress, nonce_data_availability_mode: u32)`

Changes the transaction nonce data availability mode for the given target.

## `stop_cheat_nonce_data_availability_mode`
> `fn stop_cheat_nonce_data_availability_mode(target: ContractAddress)`

Cancels the `cheat_nonce_data_availability_mode` / `start_cheat_nonce_data_availability_mode` for the given target.

## `stop_cheat_nonce_data_availability_mode_global`
> `fn stop_cheat_nonce_data_availability_mode_global()`

Cancels the `start_cheat_nonce_data_availability_mode_global`.
