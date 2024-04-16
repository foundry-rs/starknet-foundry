# `block_number`

Cheatcodes modifying `block_number`:

## `roll`
> `fn roll(target: CheatTarget, block_number: u64, span: CheatSpan)`

Changes the block number for the given target and span.

## `start_roll`
> `fn start_roll(target: CheatTarget, block_number: u64)`

Changes the block number for the given target.

# `stop_roll`
> `fn stop_roll(target: CheatTarget)`

Cancels the `roll` / `start_roll` for the given target.