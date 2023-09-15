# Fork Testing

Forge supports testing in a forked environment. Each test can fork the state of a specified real
network and perform actions on top of it.

> ⚠️ **Warning**
> 
> Calls to the nodes are not cached at the moment, so you can expect test not to be as fast
> as they can. We will improve it in the future.

There are two ways of configuring a fork:
- by specifying `url` and `block_id` parameters in the `#[fork(...)]` attribute
- or by passing fork name to the `#[frok(...)` attribute

## Configure fork in the attribute

It is possible to pass `url` and `block_id` arguments to the `fork` attribute.

```rust
use snforge_std::{ BlockTag, BlockId };

#[test]
#[fork(url: "http://your.rpc.url", block_id: BlockId::Tag(BlockTag::Latest))]
fn test_using_forked_state() {
    // ...
}
```

Once such a configuration is passed, it is possible to call any state defined on the network.

## Configure forks in the `Scarb.toml`

Although passing named arguments works fine, you have to copy-paste it each time you want to use
the same fork in tests.

Forge solves this issue by allowing fork configuration inside the `Scarb.toml` file.
```toml
[[tool.snforge.fork]]
name = "SOME_NAME"
url = "http://your.rpc.url"
block_id.tag = "Latest"

[[tool.snforge.fork]]
name = "SOME_OTHER_NAME"
url = "http://your.second.rpc.url"
block_id.tag = "Pending"
```

There are two more variants of the `block_id` field:
- `block_id.number = "1"`
- `block_id.hash = "2"`

From this moment forks can be set by name in the `fork` attribute.

```rust
#[test]
#[fork("SOME_NAME")]
fn test_using_first_fork() {
    // ...
}

#[test]
#[fork("SOME_OTHER_NAME")]
fn test_using_second_fork() {
    // ...
}
```
