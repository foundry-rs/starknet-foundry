# Project Configuration

## Cast
### Defining profiles in `Scarb.toml`

To be able to work with the network, you need to supply cast with a few parameters - namely the network name, rpc node url and an account name that should be used to interact with it. This can be done by either supplying cast with those parameters directly [see more detailed CLI description](../appendix/cast.md) or you can put them into `Scarb.toml` file:


```toml
(...)
[tool.cast.myprofile]
account = "user"
network = "testnet"
url = "http://127.0.0.1:5050/rpc"
(...)
```

With `Scarb.toml` configured this way, we can just pass `--profile myprofile` argument to make sure cast uses parameters defined in the profile.

> 📝 **Note**
> `Scarb.toml` file has to be present in current or any of the parent directories.
> Alternatively, you can also point to `Scarb.toml` path with `--path-to-scarb-toml <PATH>` flag.

> 💡 **Info**
> Not all parameters have to be present in the configuration - you can choose to include only some of them and supply the rest of them using CLI flags. You can also override parameters from the configuration using CLI flags.


```shell
$ cast --profile myprofile \
    call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```

### Multiple profiles

You can have multiple profiles defined in the `Scarb.toml`.

### Default profile

If you don't need multiple profiles, you can define the parameters without specifying one:

```toml
(...)
[tool.cast]
account = "user123"
network = "testnet"
url = "http://127.0.0.1:5050/rpc"
(...)
```

That way, you can omit passing `--profile` parameter:

```shell
$ cast call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```
