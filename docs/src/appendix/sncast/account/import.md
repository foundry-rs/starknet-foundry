# `import`
Import an account to accounts file.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## `--name, -n <NAME>`
Optional.

Name of the account to be imported.

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
Optional.

Account private key.

## `--private-key-file <PRIVATE_KEY_FILE_PATH>`
Optional. If neither `--private-key` nor `--private-key-file` is passed, the user will be prompted to enter the account private key.

Path to the file holding account private key.

## `--salt, -s <SALT>`
Optional.

Salt for the account address.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to snfoundry.toml.
