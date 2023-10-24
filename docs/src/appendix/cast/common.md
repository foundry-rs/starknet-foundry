# `sncast` common flags

## `--profile, -p <PROFILE_NAME>`
Optional.

Profile name in `Scarb.toml` config file.

## `--path-to-scarb-toml, -s <PATH>`
Optional.

Path to `Scarb.toml` file.

If supplied, cast will not look for `Scarb.toml` file in current (or parent) directory, but will use this path instead.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `Scarb.toml`.

## `--account, -a <ACCOUNT_NAME>`
Optional.

Account name used to interact with the network, aliased in open zeppelin accounts file.

Overrides account from `Scarb.toml`.

If used with `--keystore`, should be a path to [starkli account JSON file](https://book.starkli.rs/accounts#accounts).

## `--accounts-file, -f <PATH_TO_ACCOUNTS_FILE>`
Optional.

Path to the open zeppelin accounts file holding accounts info. Defaults to `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

## `--keystore, -k <PATH_TO_KEYSTORE_FILE>`
Optional.

Path to [keystore file](https://book.starkli.rs/signers#encrypted-keystores).
When specified, the --account argument must be a path to [starkli account JSON file](https://book.starkli.rs/accounts#accounts).

## `--int-format`
Optional.

If passed, values will be displayed in decimal format. Default is addresses as hex and fees as int.

## `--hex-format`
Optional.

If passed, values will be displayed in hex format. Default is addresses as hex and fees as int.

## `--json, -j`
Optional.

If passed, output will be displayed in json format.

## `--wait, -w`
Optional.

If passed, command will wait until transaction is accepted or rejected.

## `--version, -v`

Prints out `sncast` version.

## `--help, -h`

Prints out help.
