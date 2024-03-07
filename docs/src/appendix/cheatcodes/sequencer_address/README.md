# `sequencer_address`

Cheatcodes modifying `sequencer_address`:

* [`elect`](./elect.md) - changes the sequencer address for the given target, for given span
* [`start_elect`](./start_elect.md) - changes the sequencer address for the given target until [`stop_elect`](./stop_elect.md) is called
* [`stop_elect`](./stop_elect.md) - cancels the [`elect`](./elect.md) / [`start_elect`](./start_elect.md) for the given target
