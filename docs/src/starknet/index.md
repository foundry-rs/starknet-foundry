# Interacting with Starknet

Starknet Foundry cast is a command line tool for performing Starknet RPC calls. With it, You can easily declare, deploy, invoke and call Starknet contracts!

> 💡 **Info**
> At the moment, cast only supports contracts written in cairo 1.

## How to use cast

To use cast, run the cast command followed by a subcommand (see [available commands](../appendix/cast/index.html)):
```shell
$ cast <subcommand>
```

If `Scarb.toml` is present and configured with `[tool.protostar]`, `rpc_url`, `network` and `account` name will be taken from it. You can, however, overwrite their values by supplying them as flags directly to cast cli.

## Example

Let's use cast to call a contract's function:

```shell
cast --account myotheruser call --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 --function-name get --calldata 0x0 --block-id latest
command: Call
response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]

```

> 📝 **Note**
> In the above example we supply cast with `--account` flag, that overwrites `account` found in Scarb.toml (if any). We do not supply `--network` and `--url` flags, which means they must be present in Scarb.toml - otherwise an error will be returned.
