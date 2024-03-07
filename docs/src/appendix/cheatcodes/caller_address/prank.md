# `prank`

> `fn prank(target: CheatTarget, caller_address: ContractAddress, span: CheatSpan)`

Changes the caller address for the given target, for a given duration.
This change can be canceled with [`stop_prank`](./stop_prank.md).

- `target` - instance of [`CheatTarget`](../cheat_target.md) specifying which contracts to prank
- `caller_address` - caller address to be set
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the duration of prank
