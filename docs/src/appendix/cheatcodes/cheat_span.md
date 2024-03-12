# `CheatSpan`

```rust
enum CheatSpan {
    Indefinite: (),
    Calls: usize
}
```

`CheatSpan` is an enum used to specify for how long the target should be cheated.
- `Indefinite` applies the cheatcode indefinitely, until the cheat is canceled manually (e.g. using `stop_warp`).
- `Calls` applies the cheatcode for specified number of calls, after which the cheat is canceled (or until the cheat is canceled manually).
