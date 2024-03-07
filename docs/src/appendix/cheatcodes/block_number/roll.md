# `roll`

> `fn roll(target: CheatTarget, block_number: u64, span: CheatSpan)`

Changes the block number for the given target, for a given duration.
This change can be canceled with [`stop_roll`](./stop_roll.md).

- `target` - instance of [`CheatTarget`](../cheat_target.md) specifying which contracts to roll
- `block_number` - block number to be set
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the duration of roll
