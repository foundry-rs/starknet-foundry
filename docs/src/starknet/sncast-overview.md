# `sncast` Overview

Starknet Foundry `sncast` is a command line tool for performing Starknet RPC calls. With it, you can easily interact with Starknet contracts!

> 💡 **Info**
> At the moment, `sncast` only supports contracts written in [Cairo](https://github.com/starkware-libs/cairo) v1 and v2.

> ⚠️ **Warning**
> Currently, support is only provided for accounts that use the default signature based on the [Stark curve](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/stark-curve).

## How to Use `sncast`

To use `sncast`, run the `sncast` command followed by a subcommand (see [available commands](../appendix/sncast.md)):

<!-- { "ignored": true } -->
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

<!-- TODO(#2736) -->
<!-- { "ignored": true } -->
```shell
$ sncast call \
    --url http://127.0.0.1:5055 \
    --contract-address 0x522dc7cbe288037382a02569af5a4169531053d284193623948eac8dd051716 \
    --function "pokemon" \
    --arguments '"Charizard"' \
    --block-id latest
```

<details>
<summary>Output:</summary>

```shell
command: call
response: [0x0, 0x0, 0x43686172697a617264, 0x9, 0x0, 0x0, 0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf]
```
</details>
<br>

> 📝 **Note**
> In the above example we supply `sncast` with `--account` and `--url` flags. If `snfoundry.toml` is present, and have these properties set, values provided using these flags will override values from `snfoundry.toml`. Learn more about `snfoundry.toml` configuration [here](../projects/configuration.md#sncast).


### Arguments

Some `sncast` commands (namely `call`, `deploy` and `invoke`) allow passing arguments to perform an action with on the blockchain.

Under the hood cast always send request with serialized form of arguments, but it can be passed in 
human-readable form thanks to the [calldata transformation](./calldata-transformation.md) feature present in Cast.

In the example above we called a function with a deserialized argument: `'"Charizard"'`, passed using `--arguments` flag.

> ⚠️ **Warning**
> Cast will not verify the serialized calldata. Any errors caused by passing improper calldata in a serialized form will originate from the network.
> Basic static analysis is possible only when passing expressions - see [calldata transformation](./calldata-transformation.md).


### Using Serialized Calldata

The same result can be achieved by passing serialized calldata, which is a list of hexadecimal-encoded field elements.

For example, this is equivalent to using the --calldata option with the following value: 0x0 0x43686172697a617264 0x9.

To obtain the serialized form of the wished data, you can write a Cairo program that calls `Serde::serialize` on subsequent arguments and displays the results.

Read more about it in the [Cairo documentation](https://book.cairo-lang.org/appendix-03-derivable-traits.html?highlight=seri#serializing-with-serde).

### How to Use `--wait` Flag

Let's invoke a transaction and wait for it to be `ACCEPTED_ON_L2`.

<!-- { "contract_name": "HelloSncast", "ignored_output": true } -->
```shell
$ sncast --account my_account \
    --wait \
    deploy \
	--url http://127.0.0.1:5055 \
    --class-hash 0x0555d84fd95ab9fa84a826382ca91127336d4b3c640d8571c32c4e7717e38799 \
    --fee-token strk
```

<details>
<summary>Output:</summary>

```shell
Transaction hash: [..]
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
contract_address: [..]
transaction_hash: [..]

To see deployment details, visit:
contract: https://starkscan.co/search/[..]
transaction: https://starkscan.co/search/[..]
```
</details>
<br>

As you can see command waited for the transaction until it was `ACCEPTED_ON_L2`.

After setting up the `--wait` flag, command waits 60 seconds for a transaction to be received and (another not specified
amount of time) to be included in the block.

> 📝 **Note**
> By default, all commands don't wait for transactions.
