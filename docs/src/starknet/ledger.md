# Ledger Hardware Wallet

`sncast` supports Ledger hardware wallets as a signing device. Use it anywhere a signer is required, or run `sncast ledger` subcommands for device-specific operations.

## Prerequisites

`sncast` communicates with the [Starknet Ledger app](https://github.com/LedgerHQ/app-starknet) running on your device. Make sure the Starknet app is installed and open before running any `sncast ledger` commands.

> ℹ️ **Note**
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

Output:

```shell
App Version: 1.1.1
```

## Getting Public Key

Once you've decided on a path, you can read the corresponding public key from your device:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --path "m//starknet'/sncast'/0'/0'/0"
```

By default, the public key is shown on the Ledger device for manual confirmation. You can skip this confirmation with `--no-display`:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --path "m//starknet'/sncast'/0'/0'/0" --no-display
```

Output:

```shell
Public Key: 0x[..]
```

## Using Ledger as Signer

The most common use case is controlling Starknet accounts with Ledger. Pass the `--ledger-path` flag when creating or importing an account to bind the derivation path to it.

### Derivation Path Binding

When an account is created or imported with `--ledger-path`, the derivation path is saved in the accounts file (`ledger_path` field). Because of that, subsequent commands (invoke, declare, deploy, etc.) only need `--account my_ledger_account`. 

> ℹ️ **Note**
>
> `--keystore` and `--ledger-path` cannot be used together. Use one or the other.

### Creating and Deploying an Account

<!-- { "requires_ledger": true } -->
```shell
$ sncast \
    --ledger-path "m//starknet'/sncast'/0'/0'/0" \
    account create \
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

<!-- { "ignored": true } -->
```shell
$ sncast \
    account import \
    --network sepolia \
    --name my_ledger_account \
    --address 0x1 \
    --ledger-path "m//starknet'/sncast'/0'/0'/0" \
    --type oz
```

### Invoking Contracts

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

### Declaring and Deploying Contracts

<!-- { "ignored": true } -->
```shell
$ sncast \
    --account my_ledger_account \
    declare \
    --network sepolia \
    --contract-name MyContract
```

<!-- { "ignored": true } -->
```shell
$ sncast \
    --account my_ledger_account \
    deploy \
    --network sepolia \
    --class-hash 0x[..]
```

## Signing Raw Hashes

> ⚠️ **Warning**
>
> Blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes from trusted sources. If you're sending transactions, [use Ledger as a signer](#using-ledger-as-signer) instead of using this command.

You can sign a single raw hash with your Ledger device:

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --path "m//starknet'/sncast'/0'/0'/0" \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

A confirmation screen will be displayed on the device. Once approved, the signature is printed to the console.

Output:

```shell
Signature: 0x[..]
```
