# `sncast` Overview

Starknet Foundry `sncast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

> 💡 **Info**
> At the moment, `sncast` only supports contracts written in [Cairo](https://github.com/starkware-libs/cairo) v1 and v2.

> ⚠️ **Warning**
> Currently, support is only provided for accounts that use the default signature based on the [Stark curve](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/stark-curve).

## How to Use `sncast`

To use `sncast`, run the `sncast` command followed by a subcommand (see [available commands](../appendix/sncast.md)):

<!-- ignore -->
```shell
$ sncast <subcommand>
```

If `snfoundry.toml` is present and configured with `[sncast.default]`, `url`, `accounts-file` and `account` name will be taken from it.
You can, however, overwrite their values by supplying them as flags directly to `sncast` cli.

> 💡 **Info**
> Some transactions (like declaring, deploying or invoking) require paying a fee, and they must be signed.

## Examples

### General Example

Let's use `sncast` to call a contract's function:

```shell
$ sncast --account myuser \
    call \
    --url http://127.0.0.1:5050 \
    --contract-address 0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9 \
    --function get \
    --calldata 0x0 \
    --block-id latest
```

<details>
<summary>Output:</summary>

```shell
command: call
response: [0x0]
```
</details>
<br>

> 📝 **Note**
> In the above example we supply `sncast` with `--account` and `--url` flags. If `snfoundry.toml` is present, and have these properties set, values provided using these flags will override values from `snfoundry.toml`. Learn more about `snfoundry.toml` configuration [here](../projects/configuration.md#sncast).


### Calldata

Some `sncast` commands (namely `call`, `deploy` and `invoke`) allow passing *calldata* - a series of arguments to perform an action with on blockchain.

In the example above we called a function with an argument: `0x0`, passed using `--calldata` flag.

Please note the notation of the argument. The default way of passing calldata is a list of hexadecimally encoded field elements - the *serialized* calldata.
To obtain the serialized form of the wished data, one must write a Cairo program calling `Serde::serialize` on subsequent arguments and displaying the results.

It is also possible to pass calldata in more friendly, human readable form thanks to the [calldata transformation](./calldata-transformation.md) feature present in Cast.

> ⚠️ **Warning**
> Cast will not verify the serialized calldata. Any errors caused by passing improper calldata in a serialized form will originate from the network.
> Basic static analysis is possible only when passing expressions - see [calldata transformation](./calldata-transformation.md).

### How to Use `--wait` Flag

Let's invoke a transaction and wait for it to be `ACCEPTED_ON_L2`.

```shell
$ sncast --account myuser \
    --wait \
    deploy \
	--url http://127.0.0.1:5050 \
    --class-hash 0x8448a68b5ea1affc45e3fd4b8b480ea36a51dc34e337a16d2567d32d0c6f8a
```

<details>
<summary>Output:</summary>

```shell
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

To see deployment details, visit:
contract: https://starkscan.co/search/0x1d91599ec6...
transaction: https://starkscan.co/search/0x3062310a1e...
```
</details>
<br>

As you can see command waited for the transaction until it was `ACCEPTED_ON_L2`.

After setting up the `--wait` flag, command waits 60 seconds for a transaction to be received and (another not specified
amount of time) to be included in the block.

> 📝 **Note**
> By default, all commands don't wait for transactions.
