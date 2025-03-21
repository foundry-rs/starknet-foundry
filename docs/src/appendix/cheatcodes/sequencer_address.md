# `sequencer_address`

Cheatcodes modifying `sequencer_address`:

## `cheat_sequencer_address`
> `fn cheat_sequencer_address(contract_address: ContractAddress, sequencer_address: ContractAddress, span: CheatSpan)`

Changes the sequencer address for the given target and span.

## `start_cheat_sequencer_address_global`
> `fn start_cheat_sequencer_address_global(sequencer_address: ContractAddress)`

Changes the sequencer address for all targets.

## `start_cheat_sequencer_address`
> `fn start_cheat_sequencer_address(contract_address: ContractAddress, sequencer_address: ContractAddress)`

Changes the sequencer address for the given target.

## `stop_cheat_sequencer_address`
> `fn stop_cheat_sequencer_address(contract_address: ContractAddress)`

Cancels the `cheat_sequencer_address` / `start_cheat_sequencer_address` for the given target.

## `stop_cheat_sequencer_address_global`
> `fn stop_cheat_sequencer_address_global()`

Cancels the `start_cheat_sequencer_address_global`.
