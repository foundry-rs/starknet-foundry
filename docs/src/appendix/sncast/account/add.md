# `add`
Import an account to accounts file.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## Required Common Arguments — Passed By CLI or Specified in `Scarb.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <NAME>`
Required.

Name of the account to be added.

## `--address, -a <ADDRESS>`
Required.

Address of the account.

## `--class-hash, -c <CLASS_HASH>`
Optional.

Class hash of the account.

## `--deployed, -d`
Optional.

Specify account deployment status as deployed.
If not passed, sncast will check whether the account is deployed or not.

## `--private-key <PRIVATE_KEY>`
Optional. Required if `--private-key-file` is not passed.

Account private key.

## `--private-key-file <PRIVATE_KEY_FILE_PATH>`
Optional. Required if `--private-key-file` is not passed.

Path to the file holding account private key.

## `--public-key <PUBLIC_KEY>`
Optional.

Account public key.
If not passed, will be computed from `--private-key`.

## `--salt, -s <SALT>`
Optional.

Salt for the account address.

## `--add-profile`
Optional.

If passed, a profile with corresponding data will be added to Scarb.toml.
