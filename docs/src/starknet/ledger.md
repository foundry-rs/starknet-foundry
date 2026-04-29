# Ledger Hardware Wallet

`sncast` supports Ledger hardware wallets as a signing device. Use it anywhere a signer is required, or run `sncast ledger` subcommands for device-specific operations.

## Prerequisites

`sncast` communicates with the [Starknet Ledger app](https://github.com/LedgerHQ/app-starknet) running on your device (currently supported version: 2.3.4). Make sure the Starknet app is installed and open before running any `sncast ledger` commands.

> 📝 **Note**
>
> As of this writing, the latest version of the Starknet Ledger app only supports blind signing a single hash. While not ideal, it's the most secure signer available.

## Deciding on Wallet Paths

Before using the Ledger app, you must decide which wallet paths to use with your accounts.

The Starknet Ledger app requires [EIP-2645 HD paths](./eip-2645-hd-paths.md). Learn more about path management and best practices on the [EIP-2645 HD Paths](./eip-2645-hd-paths.md) page.

## Checking App Version

Checking the app version is a simple way to verify that `sncast` can communicate with your Ledger. With the Starknet app open on the device, run:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger app-version
```

<details>
<summary>Output:</summary>

```shell
App Version: 2.3.4
```
</details>
<br>

## Getting Public Key

Once you've decided on a path, you can read the corresponding public key from your device.

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --path "m//starknet'/sncast'/0'/1'/0"
```

> 📝 **Note**
>
> Wherever a derivation path is accepted, you can also pass `--account-id` (or `--ledger-account-id` for account commands) instead. An account ID `N` expands to `m//starknet'/sncast'/0'/<N>'/0`.

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --account-id 1
```

<details>
<summary>Output:</summary>

```shell
Public Key: 0x[..]
```
</details>
<br>

By default, the public key is shown on the Ledger device for manual confirmation. You can skip this confirmation with `--no-display`:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --account-id 1 --no-display
```

## Using Ledger as Signer

The most common use case is controlling Starknet accounts with Ledger. Pass `--ledger-path` or `--ledger-account-id` to `account create` or `account import` to bind the derivation path to the account.

### Derivation Path Binding

When an account is created or imported using `--ledger-path` or `--ledger-account-id`, its derivation path is stored in the accounts file (`ledger_path` field). As a result, subsequent commands like invoke, declare, or deploy only require `--account my_ledger_account`, there's no need to specify `--ledger-path` again.

### Creating and Deploying an Account

<!-- { "requires_ledger": true } -->
```shell
$ sncast \
    account create \
    --ledger-path "m//starknet'/sncast'/0'/1'/0" \
    --network sepolia \
    --name my_ledger_account
```

<!-- { "ignored": true, "requires_ledger": true } -->
```shell
$ sncast \
    account create \
    --ledger-account-id 1 \
    --network sepolia \
    --name my_ledger_account
```

This fetches the public key from the Ledger device at the specified path and calculates the account address. Prefund the address with STRK tokens, then deploy:

<!-- { "ignored": true } -->
```shell
$ sncast \
    --account my_ledger_account \
    account deploy \
    --network sepolia
```

A signing confirmation will be shown on the Ledger device. Once approved, the deployment transaction is sent.

### Importing an Existing Account

If you already have a deployed account managed by your Ledger, you can import it:

<!-- { "ignored": true, "requires_ledger": true } -->
```shell
$ sncast \
    account import \
    --network sepolia \
    --name my_ledger_account \
    --address 0x1 \
    --ledger-path "m//starknet'/sncast'/0'/1'/0" \
    --type oz
```

<!-- { "ignored": true, "requires_ledger": true } -->
```shell
$ sncast \
    account import \
    --network sepolia \
    --name my_ledger_account \
    --address 0x1 \
    --ledger-account-id 1 \
    --type oz
```

### Sending Transactions

Once an account is set up, use it with any command that requires signing. A Ledger confirmation will appear on the device for each transaction:

<!-- { "ignored": true } -->
```shell
$ sncast \
    --account my_ledger_account \
    invoke \
    --network sepolia \
    --contract-address 0x1 \
    --function "transfer" \
    --arguments '0x2, u256:100'
```

<details>
<summary>Output:</summary>

```shell
Ledger device will display a confirmation screen — approve it to continue...

Success: Invoke completed

Transaction Hash: 0x[..]

To see invocation details, visit:
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

## Signing Raw Hashes

> ⚠️ **Warning**
>
> Blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes from trusted sources. If you're sending transactions, [use Ledger as a signer](#using-ledger-as-signer) instead of using this command.

You can sign a single raw hash with your Ledger device:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --path "m//starknet'/sncast'/0'/1'/0" \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --account-id 1 \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

A confirmation screen will be displayed on the device. Once approved, the signature is printed to the console.

<details>
<summary>Output:</summary>

```shell
Hash signature:
r: 0x[..]
s: 0x[..]
```
</details>
<br>
