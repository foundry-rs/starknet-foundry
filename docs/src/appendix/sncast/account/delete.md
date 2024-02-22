# `delete`
Delete an account from `accounts-file` and its associated snfoundry profile.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name which is going to be deleted.

## `--network`
Optional.

Network in `accounts-file` associated with the account. By default, the network of rpc node.

## `--yes`
Optional.

If passed, assume "yes" as answer to confirmation prompt and run non-interactively
