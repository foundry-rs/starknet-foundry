# Integration With Devnet

[`starknet-devnet`](https://0xspaceshard.github.io/starknet-devnet/) is a local Starknet node used for development and testing. `sncast` provides inbuilt support for some of its features, making it easier to work with it.

## Predeployed Accounts

When you start `starknet-devnet`, it automatically predeploys some contracts, including set of accounts with known details (read more about them [here](https://0xspaceshard.github.io/starknet-devnet/docs/predeployed)).

You can use these accounts directly in `sncast` without needing to import them. 
They are available under specific names - `devnet-1`, `devnet-2`, ..., `devnet-<N>` (where N is the number of predeployed accounts, by default it's 10). 

> 📝 **Note**
>
> Devnet accounts can't be used together with `sepolia` and `mainnet` values for `--network` flag.


### Example

Let's invoke a contract using `devnet-1` account.

```shell
$ sncast --account devnet-1 invoke \
  --contract-address 0x0589a8b8bf819b7820cb699ea1f6c409bc012c9b9160106ddc3dacd6a89653cf \
  --function "get_balance"
```

<details>
<summary>Output:</summary>

```shell
Success: Invoke completed

Transaction Hash: [..]

To see invocation details, visit:
transaction: https://sepolia.starkscan.co/tx/[..]
```
</details>

> 📝 **Note**
>
> If you have an account named `devnet-1` (or any other predeployed account name) in your accounts file, `sncast` will prioritize using that one instead of the inbuilt devnet account.
