# Printing Current Configuration

## Overview

Sometimes, before executing any other `sncast` command, one may want to make sure that the right
configuration settings are being used (e.g., proper network or account is used).

Top see what `sncast` will use before running other commands, you can use:

- [`show-config`](#show-config) to show the **effective configuration** for the selected profile
- [`config-path`](#config-path) to show **actual files** that contribute to effective config

## `show-config`

Replace any subcommand (and its parameters) with `show-config` to print the effective configuration.

### Example

```shell
$ sncast \
  --profile default \
  --account my_account \
  show-config 
```

<details>
<summary>Output:</summary>

```shell
Chain ID:            alpha-sepolia
RPC URL:             http://127.0.0.1:5055/rpc
Account:             my_account
Accounts File Path:  [..]/accounts.json
Wait Timeout:        300s
Wait Retry Interval: 5s
Show Explorer Links: true
```
</details>
<br>

## `config-path`

```shell
$ sncast config-path
```

<details>
<summary>Output:</summary>

```shell
Local Config:  [..]snfoundry.toml
Global Config: [..]snfoundry.toml
```
</details>
<br>

See [Project Configuration](../projects/configuration.md#sncast) for how local and global configs are combined.
