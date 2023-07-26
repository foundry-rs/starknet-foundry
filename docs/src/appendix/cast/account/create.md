# `create`
Create all data required for account deployment.

## Required common arguments - passed by value or specified in Scarb.toml

* [`network`](../common.md#--network--n-network_name)
* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name under which account information is going to be saved.

## `--salt, -s <SALT>`
Optional.

Salt for the account address. If omitted random one will be used.

## `--add-profile, -a`
Optional.

If passed, a profile with corresponding data will be created in Scarb.toml
