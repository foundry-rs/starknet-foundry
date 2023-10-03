# `delete`
Delete an account from `accounts-file` and optionally his associated Scarb profile.

## Required common arguments - passed by CLI or specified in Scarb.toml

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name which is going to be deleted.

## `--delete-profile`
Optional.

If passed, his associated profile will be removed from Scarb.toml.

## `--network`
Optional.

Network in `accounts-file` associated with the account.
