# Fork Testing

`snforge` supports testing in a forked environment. Each test can fork the state of a specified real
network and perform actions on top of it.

> 📝 **Note**
>
> Actions are performed on top of the `forked` state which means real network is not affected.

## Fork Configuration

There are two ways of configuring a fork:
- by specifying `url` and `block_id` parameters in the `#[fork(...)]` attribute
- or by passing a fork name defined in your `Scarb.toml` to the `#[fork(...)]` attribute

### Configure a Fork in the Attribute

It is possible to pass `url` and `block_id` arguments to the `fork` attribute:
- `url` - RPC URL (short string)
- `block_id` - id of block which will be pin to fork (`BlockId` enum)

```rust
enum BlockId {
    Tag: BlockTag,
    Hash: felt252,
    Number: u64,
}

enum BlockTag {
    Latest,
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

### Configure Fork in `Scarb.toml`

Although passing named arguments works fine, you have to copy-paste it each time you want to use
the same fork in tests.

`snforge` solves this issue by allowing fork configuration inside the `Scarb.toml` file.
```toml
[[tool.snforge.fork]]
name = "SOME_NAME"
url = "http://your.rpc.url"
block_id.tag = "Latest"

[[tool.snforge.fork]]
name = "SOME_SECOND_NAME"
url = "http://your.second.rpc.url"
block_id.number = "123"

[[tool.snforge.fork]]
name = "SOME_THIRD_NAME"
url = "http://your.third.rpc.url"
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

## Testing Forked Contracts

Once the fork is configured, the test will run on top of the forked state, meaning that it will have access to every contract deployed on the real network.

With that, you can now interact with any contract from the chain [the same way you would in a standard test](../testing/contracts.md).

> ⚠️ **Warning**
> 
> The following cheatcodes won't work for forked contracts written in **Cairo 0**:
>
> - start_spoof / stop_spoof
> - spy_events
>
