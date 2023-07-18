# Cast overview

Starknet Foundry `cast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

> 💡 **Info**
> At the moment, `cast` only supports contracts written in Cairo 1 and Cairo 2.

## How to use cast

To use cast, run the cast command followed by a subcommand (see [available commands](../appendix/cast.md)):
```shell
$ cast <subcommand>
```

If `Scarb.toml` is present and configured with `[tool.cast]`, `url`, `network` and `account` name will be taken from it. You can, however, overwrite their values by supplying them as flags directly to `cast` cli.

> 💡 **Info**
> Some transactions (like declaring, deploying or invoking) require paying a fee, and they must be signed.

## Example

Let's use `cast` to call a contract's function:

```shell
$ cast --account myotheruser \
    --network testnet \
    --url http://127.0.0.1:5050/rpc \
    call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]
```

> 📝 **Note**
> In the above example we supply cast with `--account`, `--network` and `--url` flags. If `Scarb.toml` is present, and have these properties set, values provided using these flags will override values from `Scarb.toml`. Learn more about `Scarb.toml` configuration [here](../projects/configuration.md#cast).
