# `CheatSpan`

```rust
enum CheatSpan {
    Indefinite: (),
    TargetCalls: NonZero<usize>
}
```

`CheatSpan` is an enum used to specify for how long the target should be cheated for.

- `Indefinite` applies the cheatcode indefinitely, until the cheat is canceled manually (e.g. using `stop_cheat_block_timestamp`).
- `TargetCalls` applies the cheatcode for a specified number of calls to the target, after which the cheat is canceled (or until the cheat is canceled manually).
