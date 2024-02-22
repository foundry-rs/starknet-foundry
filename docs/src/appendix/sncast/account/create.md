# `create`
Prepare all prerequisites for account deployment.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name under which account information is going to be saved.

## `--salt, -s <SALT>`
Optional.

Salt for the account address. If omitted random one will be generated.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to snfoundry.toml.

## `--class-hash, -c`
Optional.

Class hash of a custom openzeppelin account contract declared to the network.
