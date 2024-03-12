# `elect`

> `fn elect(target: CheatTarget, sequencer_address: ContractAddress, span: CheatSpan)`

Changes the sequencer address for the given target and span.
This change can be canceled with [`stop_elect`](./stop_elect.md).

- `target` - instance of [`CheatTarget`](../cheat_target.md) specifying which contracts to elect
- `sequencer_address` - sequencer address to be set
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the span of elect
