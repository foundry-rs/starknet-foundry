# `get_call_trace`

```rust
fn get_call_trace() -> CallTrace;
```

(For whole structure definition, please refer
to [`snforge-std` source](https://github.com/foundry-rs/starknet-foundry/tree/v0.16.0/snforge_std))

Gets current call trace of the test, up to the last call made to a contract.

The whole structure is represented as a tree of calls, in which each contract interaction
is a new execution scope - thus resulting in a new nested trace.

> 📝 **Note**
>
> The topmost-call is representing the test call, which will always be present if you're running a test.

## Displaying the trace

The `CallTrace` structure implements a `Display` trait, for a pretty-print with indentations

```rust
println!("{}", get_call_trace());
```