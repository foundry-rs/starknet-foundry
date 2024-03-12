# `warp`

> `fn warp(target: CheatTarget, block_timestamp: u64, span: CheatSpan)`

Changes the block timestamp for the given target and span.
This change can be canceled with [`stop_warp`](./stop_warp.md).

- `target` - instance of [`CheatTarget`](../cheat_target.md) specifying which contracts to warp
- `block_timestamp` - block timestamp to be set
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the span of warp
