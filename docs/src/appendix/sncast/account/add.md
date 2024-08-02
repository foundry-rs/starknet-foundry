# `add`
Import an account to accounts file.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## `--name, -n <NAME>`
Required.

Name of the account to be added.

## `--address, -a <ADDRESS>`
Required.

Address of the account.

## `--type, -t <ACCOUNT_TYPE>`
Required.

Type of the account. Possible values: oz, argent, braavos.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--class-hash, -c <CLASS_HASH>`
Optional.

Class hash of the account.

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

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to snfoundry.toml.
