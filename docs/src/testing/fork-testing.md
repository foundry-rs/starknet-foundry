# Fork Testing

Forge supports testing in a forked environment. Each test can fork the state of a specified real
network and perform actions on top of it.

> 📝 **Note**
>
> Actions are performed on to of the `forked` state which means real network is not affected.


> ⚠️ **Warning**
>
> Calls to the nodes are not cached between test cases at the moment, so you can expect fork
> tests to be a bit slower than regular ones. Optimisations are well on their way though!

There are two ways of configuring a fork:
- by specifying `url` and `block_id` parameters in the `#[fork(...)]` attribute
- or by passing fork name defined in your `Scarb.toml` to the `#[fork(...)` attribute

## Configure a Fork in the attribute

It is possible to pass `url` and `block_id` arguments to the `fork` attribute:
- `url` - short string
- `block_id` - `BlockId` enum

```rust
enum BlockTag {
    Latest,
    Pending,
}

enum BlockId {
    Tag: BlockTag,
    Hash: felt252,
    Number: felt252,
}
```


```rust
use snforge_std::BlockId;

#[test]
#[fork(url: "http://your.rpc.url", block_id: BlockId::Number(123))]
fn test_using_forked_state() {
    // ...
}
```

Once such a configuration is passed, it is possible to use state and contracts defined on the specified network.

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
name = "SOME_SECOND_NAME"
url = "http://your.second.rpc.url"
block_id.tag = "Pending"

[[tool.snforge.fork]]
name = "SOME_THIRD_NAME"
url = "http://your.third.rpc.url"
block_id.number = "123"

[[tool.snforge.fork]]
name = "SOME_FOURTH_NAME"
url = "http://your.fourth.rpc.url"
block_id.hash = "0x123"
```

From this moment forks can be set using their name in the `fork` attribute.

```rust
#[test]
#[fork("SOME_NAME")]
fn test_using_first_fork() {
    // ...
}

#[test]
#[fork("SOME_SECOND_NAME")]
fn test_using_second_fork() {
    // ...
}

// ...
```
