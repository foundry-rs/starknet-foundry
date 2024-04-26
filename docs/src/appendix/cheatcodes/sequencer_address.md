# `sequencer_address`

Cheatcodes modifying `sequencer_address`:

## `elect`

> `fn elect(target: CheatTarget, sequencer_address: ContractAddress, span: CheatSpan)`

Changes the sequencer address for the given target and span.

## `start_elect`

> `fn start_elect(target: CheatTarget, sequencer_address: ContractAddress)`

Changes the sequencer address for a given target.

## `stop_elect`

> `fn stop_elect(target: CheatTarget)`

Cancels the `elect` / `start_elect` for the given target.
