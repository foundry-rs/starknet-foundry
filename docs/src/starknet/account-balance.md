# Account Balance

`sncast` allows you to check the balance of your account using the `sncast get balance` command.

## Basic Example

```shell
$ sncast --account my_account get balance --network devnet
```

<details>
<summary>Output:</summary>

```shell
Balance: [..] fri
```
</details>

By default, it shows the balance in STRK tokens. Other possible tokens can be specified using the `--token` flag, read more [here](../appendix/sncast/get/balance.html#--token--t-token).

## Checking Balance of a Custom Token

You can check the balance of a custom token by providing the `--token-address` flag followed by the token's contract address.

> 📝 **Note**
>
> Token address must be a valid ERC-20 token contract.

<!-- { "ignored": true } -->
```shell
$ sncast --account user1 get balance \
    --token-address <YOUR_TOKEN_ADDRESS> \
    --network sepolia
```

<details>
<summary>Output:</summary>

```shell
Balance: [..]
```
</details>

Read more about `sncast get balance` command [here](../appendix/sncast/get/balance.md).