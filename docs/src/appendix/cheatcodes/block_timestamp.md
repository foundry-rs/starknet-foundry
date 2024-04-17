# `block_timestamp`

Cheatcodes modifying `block_timestamp`:

## `warp`

> `fn warp(target: CheatTarget, block_timestamp: u64, span: CheatSpan)`

Changes the block timestamp for the given target and span.

## `start_warp`
> `fn start_warp(target: CheatTarget, block_timestamp: u64)`

Changes the block timestamp for the given target.

## `stop_warp`

> `fn stop_warp(target: CheatTarget)`

Cancels the `warp` / `start_warp` for the given target.
