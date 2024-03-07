# `tx_info`

Cheatcodes modifying `tx_info`:

* [`spoof`](./spoof.md) - changes `TxInfo` returned by `get_tx_info()` for the given target, for given span
* [`start_spoof`](./start_spoof.md) - Changes `TxInfo` returned by `get_tx_info()` for the given target until [`stop_spoof`](./stop_spoof.md) is called
* [`stop_spoof`](./stop_spoof.md) - cancels the [`spoof`](./spoof.md) / [`start_spoof`](./start_spoof.md) for the given target
