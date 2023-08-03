# Cast overview

Starknet Foundry `sncast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

> 💡 **Info**
> At the moment, `sncast` only supports contracts written in Cairo 1 and Cairo 2.

## How to use cast

To use cast, run the `sncast` command followed by a subcommand (see [available commands](../appendix/cast.md)):
```shell
$ sncast <subcommand>
```

If `Scarb.toml` is present and configured with `[tool.sncast]`, `url`, `network` and `account` name will be taken from it.
You can, however, overwrite their values by supplying them as flags directly to `sncast` cli.

> 💡 **Info**
> Some transactions (like declaring, deploying or invoking) require paying a fee, and they must be signed.

## Examples

### General example

Let's use `sncast` to call a contract's function:

```shell
$ sncast --account myuser \
    --network testnet \
    --url http://127.0.0.1:5050 \
    call \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest

command: call
response: [0x0]
```

> 📝 **Note**
> In the above example we supply cast with `--account`, `--network` and `--url` flags. If `Scarb.toml` is present, and have these properties set, values provided using these flags will override values from `Scarb.toml`. Learn more about `Scarb.toml` configuration [here](../projects/configuration.md#cast).

### How to use `--wait` flag

Let's invoke a transaction and wait for it to be `ACCEPTED_ON_L2`.

```shell
$ sncast --account myuser \
    --network testnet \
    --url http://127.0.0.1:5050 \
    --wait \
    deploy \
    --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
   
Transaction hash: 0x3062310a1e40d4b66d8987ba7447d1c7317381d0295d62cb12f2fe3f11e6983
Waiting for transaction to be received. Retries left: 11
Waiting for transaction to be received. Retries left: 10
Waiting for transaction to be received. Retries left: 9
Waiting for transaction to be received. Retries left: 8
Waiting for transaction to be received. Retries left: 7
Received transaction. Status: Pending
Received transaction. Status: Pending
Received transaction. Status: Pending
Received transaction. Status: Pending
Received transaction. Status: Pending
Received transaction. Status: Pending
command: deploy
contract_address: 0x1d91599ec661e97fdcbb10c642a1c4f920986f1a7a9659d157d0db09baaa29e
transaction_hash: 0x3062310a1e40d4b66d8987ba7447d1c7317381d0295d62cb12f2fe3f11e6983
```

As you can see command waited for the transaction until it was `ACCEPTED_ON_L2`.

After setting up the `--wait` flag, command waits 60 seconds for a transaction to be received and (another not specified
amount of time) to be included in the block.

> 📝 **Note**
> By default, all commands don't wait for transactions.
