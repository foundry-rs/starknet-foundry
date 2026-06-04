# Stateful Invariant Testing Pattern

Many protocol bugs only surface after a particular sequence of external calls. A vault balance can drift after a deposit followed by two withdrawals of the same amount; an access-control flag can flip if a privileged setter is called between two user actions. Random per-argument fuzzing (the existing `#[fuzzer]` mode) does not exercise these orderings on its own.

This page documents a small user-land pattern for running call sequences against a contract under test and asserting a global property between every transition, using primitives `snforge` already exposes.

> ℹ️ **Roadmap**
> Native support for invariant and differential testing is tracked in [#2464](https://github.com/foundry-rs/starknet-foundry/issues/2464). The recipe below is a way to do this in user-land against today's `snforge`.

## The recipe

Pick a single fuzzer-supported scalar (a `u256` works well) as the seed. Slice it into per-step action bytes. For each step:

1. Decode the byte into an action selector (which method to call) and per-call arguments.
2. Perform one external call against the contract under test.
3. Reconcile against a test-side ledger that mirrors what the contract should look like under that call.
4. Assert the invariant between the contract's reported state and the ledger.

The fuzzer runs the test many times against random seeds; each seed unfolds a different sequence of calls. A drift between contract and ledger after any transition is a violation.

## Worked example

The test:

```rust
{{#include ../../listings/invariant_testing_pattern/tests/lib.cairo}}
```

The contract under test is a minimal `Vault` with `deposit`, `withdraw`, and a `balance()` view:

```rust
{{#include ../../listings/invariant_testing_pattern/src/lib.cairo}}
```

Run with:

```shell
$ snforge test
```

A regression injected on either branch (for example, a `withdraw` that decrements by `amount + 1` instead of `amount`) drifts the ledger and is flagged within the first dozen seeds. Increase `runs:` on the `#[fuzzer]` attribute for higher coverage.

## Rotating callers

When the invariant depends on which actor performed each call (admin vs user), wrap the dispatch in a [`start_cheat_caller_address`](./../testing/using-cheatcodes.md) / `stop_cheat_caller_address` pair. Bit-pack the caller selector into the same action byte that selects the method.

## Limits of this pattern

This is a user-land workaround, not a replacement for native invariant testing. The fuzzer still generates only scalar seeds; decoding the call sequence is the test author's job. Action shrinking and per-property statistics are not provided. Issue [#2464](https://github.com/foundry-rs/starknet-foundry/issues/2464) tracks the design for native support.
