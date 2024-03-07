# `spoof`

> `fn spoof(target: CheatTarget, tx_info_mock: TxInfoMock, span: CheatSpan)`

Changes `TxInfo` returned by `get_tx_info()` for the targeted contract, for a given duration.
This change can be canceled with [`stop_spoof`](./stop_spoof.md).

- `target` - instance of [`CheatTarget`](../cheat_target.md) specifying which contracts to spoof
- `tx_info_mock` - a struct with same structure as `TxInfo` (returned by `get_tx_info()`)
- `span` - instance of [`CheatSpan`](../cheat_span.md) specifying the duration of spoof
