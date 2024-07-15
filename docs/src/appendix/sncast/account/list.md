# `list`
List all available accounts.

Account information will be retrieved from the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.
Hides user's private keys by default.

> ⚠️ **Warning**
> This command outputs cryptographic information about accounts, e.g. user's private key.
> Use it responsibly to not cause any vulnerabilities to your environment and confidential data.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--display-private-keys`, `-p`
Optional.

If passed, show private keys along with the rest of the account information.

